//! Sovereign Outbound Webhook Delivery Engine (`frappe-framework::webhook`).
//!
//! Implements GitHub-grade webhook signing, payload delivery headers,
//! HMAC-SHA256 signature calculation, subscription management,
//! persistent outbox status modeling, retry DLQ policies, and async HTTP dispatch.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Webhook errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WebhookError {
    #[error("Invalid secret key")]
    InvalidSecret,
    #[error("Serialization failure: {0}")]
    Serialization(String),
    #[error("Signature mismatch")]
    SignatureMismatch,
    #[error("Network dispatch error: {0}")]
    Network(String),
}

/// Webhook outbox lifecycle delivery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    Queued,
    Delivered,
    Failed,
    Dead,
}

impl OutboxStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Delivered => "Delivered",
            Self::Failed => "Failed",
            Self::Dead => "Dead",
        }
    }
}

/// Webhook event subscription record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookSubscription {
    pub id: CompactString,
    pub event: CompactString, // e.g. "document.submitted", "invoice.paid"
    pub target_url: CompactString,
    pub secret: CompactString,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Payload sent to subscriber endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookPayload {
    pub event: CompactString,
    pub doctype: CompactString,
    pub doc_name: CompactString,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Outbox entry for persistent database queuing and Dead-Letter Queue (DLQ) processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookOutboxEntry {
    pub id: CompactString,
    pub subscription_id: CompactString,
    pub target_url: CompactString,
    pub secret: CompactString,
    pub event: CompactString,
    pub payload_json: String,
    pub status: OutboxStatus,
    pub attempts: u32,
    pub max_retries: u32,
    pub next_retry_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_error: Option<String>,
}

impl WebhookOutboxEntry {
    /// Creates a new queued outbox entry.
    #[must_use]
    pub fn new(subscription: &WebhookSubscription, payload: &WebhookPayload) -> Result<Self, WebhookError> {
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| WebhookError::Serialization(e.to_string()))?;
        let now = Utc::now();
        let id = format!("wh_out_{}_{}", subscription.id, now.timestamp_millis());

        Ok(Self {
            id: id.into(),
            subscription_id: subscription.id.clone(),
            target_url: subscription.target_url.clone(),
            secret: subscription.secret.clone(),
            event: payload.event.clone(),
            payload_json,
            status: OutboxStatus::Queued,
            attempts: 0,
            max_retries: 5,
            next_retry_at: now,
            created_at: now,
            last_error: None,
        })
    }

    /// Computes the exponential backoff duration (50ms -> 100ms -> 200ms -> 400ms -> 800ms).
    #[must_use]
    pub fn compute_backoff(&self) -> Duration {
        let base_ms = 50u64;
        let factor = 2u64.saturating_pow(self.attempts);
        Duration::from_millis(base_ms.saturating_mul(factor))
    }

    /// Records a failed delivery attempt and advances the backoff timer.
    pub fn record_failure(&mut self, err_msg: String) {
        self.attempts = self.attempts.saturating_add(1);
        self.last_error = Some(err_msg);
        if self.attempts >= self.max_retries {
            self.status = OutboxStatus::Dead;
        } else {
            self.status = OutboxStatus::Failed;
            let delay = self.compute_backoff();
            let millis = delay.as_millis().min(i64::MAX as u128) as i64;
            self.next_retry_at = Utc::now() + chrono::Duration::milliseconds(millis);
        }
    }

    /// Marks the entry as successfully delivered.
    pub fn record_success(&mut self) {
        self.status = OutboxStatus::Delivered;
        self.last_error = None;
    }
}

/// Webhook Dispatcher & Verifier.
pub struct WebhookDispatcher;

impl WebhookDispatcher {
    /// Computes the HMAC-SHA256 signature for a payload body (`sha256=<hex>`).
    pub fn compute_signature(secret: &str, body: &[u8]) -> Result<String, WebhookError> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| WebhookError::InvalidSecret)?;
        mac.update(body);
        let tag = mac.finalize();
        Ok(format!("sha256={}", hex::encode(tag.into_bytes())))
    }

    /// Computes raw hex HMAC-SHA256 signature.
    pub fn compute_raw_hex(secret: &str, body: &[u8]) -> Result<String, WebhookError> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| WebhookError::InvalidSecret)?;
        mac.update(body);
        let tag = mac.finalize();
        Ok(hex::encode(tag.into_bytes()))
    }

    /// Verifies an incoming webhook signature against the secret and raw body.
    /// Supports `sha256=<hex>`, `v1=<hex>`, Stripe `t=...,v1=...`, or raw `<hex>`.
    #[must_use]
    pub fn verify_signature(secret: &str, body: &[u8], signature_hdr: &str) -> bool {
        let clean_hdr = signature_hdr.trim();
        if clean_hdr.is_empty() {
            return false;
        }

        // Check Stripe format: t=123456,v1=hex
        if clean_hdr.contains("v1=") {
            let mut timestamp = "";
            let mut v1_sig = "";
            for part in clean_hdr.split(',') {
                let part = part.trim();
                if let Some(t) = part.strip_prefix("t=") {
                    timestamp = t;
                } else if let Some(v1) = part.strip_prefix("v1=") {
                    v1_sig = v1;
                }
            }

            if !v1_sig.is_empty() {
                // If timestamp is present, hash `${t}.${body}`
                let payload_to_hash = if !timestamp.is_empty() {
                    let mut p = Vec::with_capacity(timestamp.len() + 1 + body.len());
                    p.extend_from_slice(timestamp.as_bytes());
                    p.push(b'.');
                    p.extend_from_slice(body);
                    p
                } else {
                    body.to_vec()
                };

                if let Ok(expected_hex) = Self::compute_raw_hex(secret, &payload_to_hash) {
                    if expected_hex.eq_ignore_ascii_case(v1_sig) {
                        return true;
                    }
                }
            }
        }

        // Check standard sha256=<hex> or raw hex
        if let Ok(expected_sig) = Self::compute_signature(secret, body) {
            if clean_hdr.eq_ignore_ascii_case(&expected_sig) {
                return true;
            }
            if let Some(raw_expected) = expected_sig.strip_prefix("sha256=") {
                if clean_hdr.eq_ignore_ascii_case(raw_expected) {
                    return true;
                }
                if let Some(hdr_hex) = clean_hdr.strip_prefix("sha256=") {
                    if raw_expected.eq_ignore_ascii_case(hdr_hex) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Prepares signed delivery headers for an HTTP POST request.
    pub fn prepare_delivery_headers(
        subscription: &WebhookSubscription,
        payload: &WebhookPayload,
    ) -> Result<(String, Vec<(String, String)>), WebhookError> {
        let json_bytes =
            serde_json::to_vec(payload).map_err(|e| WebhookError::Serialization(e.to_string()))?;
        let signature = Self::compute_signature(&subscription.secret, &json_bytes)?;

        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-RustNext-Event".to_string(), payload.event.to_string()),
            (
                "X-RustNext-Delivery".to_string(),
                format!("{}-{}", subscription.id, Utc::now().timestamp_millis()),
            ),
            ("X-RustNext-Signature-256".to_string(), signature),
        ];

        let body_str = String::from_utf8(json_bytes)
            .map_err(|e| WebhookError::Serialization(e.to_string()))?;

        Ok((body_str, headers))
    }

    /// Asynchronously dispatches a webhook delivery attempt via HTTP POST.
    pub async fn dispatch_http(
        target_url: &str,
        secret: &str,
        event: &str,
        delivery_id: &str,
        body_json: &str,
        timeout: Duration,
    ) -> Result<u16, WebhookError> {
        let signature = Self::compute_signature(secret, body_json.as_bytes())?;
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| WebhookError::Network(e.to_string()))?;

        let res = client
            .post(target_url)
            .header("Content-Type", "application/json")
            .header("X-RustNext-Event", event)
            .header("X-RustNext-Delivery", delivery_id)
            .header("X-RustNext-Signature-256", signature)
            .body(body_json.to_string())
            .send()
            .await
            .map_err(|e| WebhookError::Network(e.to_string()))?;

        Ok(res.status().as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signing_and_verification() {
        let secret = "whsec_super_secret_enterprise_key_2026";
        let sub = WebhookSubscription {
            id: "wh_sub_01".into(),
            event: "invoice.paid".into(),
            target_url: "https://api.partner.com/webhooks".into(),
            secret: secret.into(),
            is_active: true,
            created_at: Utc::now(),
        };

        let payload = WebhookPayload {
            event: "invoice.paid".into(),
            doctype: "Sales Invoice".into(),
            doc_name: "ACC-INV-2026-0001".into(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "grand_total": 45000.0,
                "status": "Paid"
            }),
        };

        let (body, headers) =
            WebhookDispatcher::prepare_delivery_headers(&sub, &payload).expect("Headers");
        let sig_hdr = headers
            .iter()
            .find(|(k, _)| k == "X-RustNext-Signature-256")
            .map(|(_, v)| v.as_str())
            .expect("Sig header");

        assert!(sig_hdr.starts_with("sha256="));
        assert!(WebhookDispatcher::verify_signature(
            secret,
            body.as_bytes(),
            sig_hdr
        ));
        assert!(!WebhookDispatcher::verify_signature(
            "wrong_secret",
            body.as_bytes(),
            sig_hdr
        ));
    }

    #[test]
    fn test_stripe_signature_verification() {
        let secret = "whsec_stripe_test_secret";
        let body = b"{\"id\":\"evt_123\",\"type\":\"payment_intent.succeeded\"}";
        let timestamp = "1727600000";

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        let v1_hex = hex::encode(mac.finalize().into_bytes());

        let sig_header = format!("t={timestamp},v1={v1_hex}");
        assert!(WebhookDispatcher::verify_signature(
            secret,
            body,
            &sig_header
        ));
    }

    #[test]
    fn test_outbox_exponential_backoff_and_dlq() {
        let sub = WebhookSubscription {
            id: "sub_1".into(),
            event: "order.created".into(),
            target_url: "https://example.com".into(),
            secret: "sec".into(),
            is_active: true,
            created_at: Utc::now(),
        };
        let payload = WebhookPayload {
            event: "order.created".into(),
            doctype: "Sales Order".into(),
            doc_name: "SO-001".into(),
            timestamp: Utc::now(),
            data: serde_json::json!({}),
        };

        let mut outbox = WebhookOutboxEntry::new(&sub, &payload).unwrap();
        assert_eq!(outbox.status, OutboxStatus::Queued);
        assert_eq!(outbox.compute_backoff(), Duration::from_millis(50));

        outbox.record_failure("HTTP 500".into());
        assert_eq!(outbox.status, OutboxStatus::Failed);
        assert_eq!(outbox.attempts, 1);
        assert_eq!(outbox.compute_backoff(), Duration::from_millis(100));

        outbox.record_failure("HTTP 502".into());
        outbox.record_failure("HTTP 503".into());
        outbox.record_failure("HTTP 504".into());
        outbox.record_failure("HTTP 500".into());

        assert_eq!(outbox.status, OutboxStatus::Dead);
        assert_eq!(outbox.attempts, 5);
    }
}
