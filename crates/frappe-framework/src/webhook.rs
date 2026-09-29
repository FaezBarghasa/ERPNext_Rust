//! Sovereign Outbound Webhook Delivery Engine (`frappe-framework::webhook`).
//!
//! Implements GitHub-grade webhook signing, payload delivery headers,
//! HMAC-SHA256 signature calculation, and subscription management.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
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

    /// Verifies an incoming webhook signature against the secret and raw body.
    #[must_use]
    pub fn verify_signature(secret: &str, body: &[u8], signature_hdr: &str) -> bool {
        let expected_sig = match Self::compute_signature(secret, body) {
            Ok(s) => s,
            Err(_) => return false,
        };
        expected_sig.trim() == signature_hdr.trim()
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
}
