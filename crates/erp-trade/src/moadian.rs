//! Iranian National Tax Administration (INTA / سامانه مودیان) E-Invoicing Engine (`erp-trade::moadian`).
//!
//! Provides canonical Iranian Tax Administration compliance:
//! - 22-character unique Tax ID (شماره منحصر به فرد مالیاتی) generation with Verhoeff/checksum verification
//! - Moadian Pattern 1 (فروش کالا و خدمات) Header & Item line standardization
//! - Automated VAT calculation (10% standard rate) and discount proration
//! - Cryptographic canonical JSON SHA-256 payload digest for INTA API submission.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

/// Moadian e-invoicing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MoadianError {
    #[error("Fiscal Memory ID must be exactly 6 characters, got '{0}'")]
    InvalidMemoryId(String),
    #[error("Stuff/Service Code (sstid) must be valid 13-digit code, got '{0}'")]
    InvalidStuffServiceCode(CompactString),
    #[error("Seller Tax ID (tins) is required")]
    MissingSellerTaxId,
    #[error("Invoice contains no item lines")]
    EmptyInvoice,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Signing error: {0}")]
    SigningError(String),
    #[error("Verification error: {0}")]
    VerificationError(String),
}

/// Invoice subject status (ins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoadianInvoiceStatus {
    Main = 1,
    Corrective = 2,
    Revocation = 3,
    Return = 4,
}

/// Unique 22-character Tax ID generator for Iranian Tax Administration.
pub struct MoadianTaxIdGenerator;

impl MoadianTaxIdGenerator {
    /// Computes the 22nd check digit using Modulo 11 check on character codes.
    #[must_use]
    pub fn compute_check_digit(input21: &str) -> char {
        let mut sum: u32 = 0;
        for (i, c) in input21.chars().enumerate() {
            let weight = (i as u32 % 8) + 2;
            let val = c as u32;
            sum += val * weight;
        }
        let remainder = sum % 11;
        if remainder < 10 {
            char::from_digit(remainder, 10).unwrap_or('0')
        } else {
            'A'
        }
    }

    /// Verifies if a 22-character Tax ID has a valid Modulo 11 check digit.
    #[must_use]
    pub fn verify_tax_id(tax_id: &str) -> bool {
        if tax_id.len() != 22 {
            return false;
        }
        let raw_21 = &tax_id[0..21];
        let expected = Self::compute_check_digit(raw_21);
        tax_id.ends_with(expected)
    }

    /// Generates canonical 22-character Tax ID:
    /// - Memory ID: 6 alphanumeric characters
    /// - Days from base epoch (2014-03-21): 5 hex characters
    /// - Sequence number: 10 hex characters
    /// - Control digit: 1 character
    pub fn generate_tax_id(
        fiscal_memory_id: &str,
        posting_datetime: DateTime<Utc>,
        sequence_num: u64,
    ) -> Result<CompactString, MoadianError> {
        if fiscal_memory_id.len() != 6 {
            return Err(MoadianError::InvalidMemoryId(fiscal_memory_id.to_string()));
        }

        // Epoch: 2014-03-21 (Start of modern Iranian tax reform base)
        let epoch = DateTime::from_timestamp(1395360000, 0).unwrap_or(posting_datetime);
        let days = (posting_datetime - epoch).num_days().max(0) as u32;

        let memory_part = fiscal_memory_id.to_uppercase();
        let days_part = format!("{days:05X}");
        let seq_part = format!("{sequence_num:010X}");

        let raw_21 = format!("{memory_part}{days_part}{seq_part}");
        let check_digit = Self::compute_check_digit(&raw_21);

        Ok(format!("{raw_21}{check_digit}").into())
    }
}

/// Moadian Invoice Line Item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoadianItemLine {
    /// Stuff / Service Code (شناسه کالا و خدمت - 13 digits)
    pub sstid: CompactString,
    /// Description
    pub sstt: CompactString,
    /// Quantity (تعداد / مقدار)
    pub am: Decimal,
    /// Unit Fee (مبلغ واحد)
    pub fee: Decimal,
    /// Price before discount (مبلغ قبل از تخفیف)
    pub prdis: Decimal,
    /// Discount amount (مبلغ تخفیف)
    pub dis: Decimal,
    /// Price after discount (مبلغ پس از تخفیف)
    pub adis: Decimal,
    /// VAT rate percentage (نرخ مالیات بر ارزش افزوده - e.g. 10.0)
    pub vra: Decimal,
    /// VAT amount (مبلغ مالیات بر ارزش افزوده)
    pub vam: Decimal,
    /// Total item line amount (مبلغ کل کالا/خدمت)
    pub tsstam: Decimal,
}

impl MoadianItemLine {
    pub fn new(
        sstid: impl Into<CompactString>,
        sstt: impl Into<CompactString>,
        qty: Decimal,
        fee: Decimal,
        discount: Decimal,
        vat_rate: Decimal,
    ) -> Self {
        let prdis = qty * fee;
        let dis = discount.min(prdis);
        let adis = prdis - dis;
        let vam = ((adis * vat_rate) / dec!(100.0)).round_dp(2);
        let tsstam = adis + vam;

        Self {
            sstid: sstid.into(),
            sstt: sstt.into(),
            am: qty,
            fee,
            prdis,
            dis,
            adis,
            vra: vat_rate,
            vam,
            tsstam,
        }
    }
}

/// Moadian Invoice Header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoadianHeader {
    pub taxid: CompactString,
    pub indatim: i64, // Milliseconds since epoch
    pub inty: u32,    // 1: Type 1 (with buyer info), 2: Type 2 (consumer)
    pub inno: CompactString,
    pub inp: u32, // 1: Sales Pattern
    pub ins: u32, // Invoice status (1: Main)
    pub tins: CompactString,
    pub bid: Option<CompactString>,
    pub tinb: Option<CompactString>,
    pub tprdis: Decimal,
    pub tdis: Decimal,
    pub tadis: Decimal,
    pub tvam: Decimal,
    pub todam: Decimal,
    pub tbill: Decimal,
}

/// Parameters for constructing a Moadian e-invoice.
#[derive(Debug, Clone)]
pub struct MoadianInvoiceParams<'a> {
    pub taxid: &'a str,
    pub inno: &'a str,
    pub seller_tax_id: &'a str,
    pub buyer_national_id: Option<&'a str>,
    pub buyer_tax_id: Option<&'a str>,
    pub posting_datetime: DateTime<Utc>,
    pub status: MoadianInvoiceStatus,
    pub items: Vec<MoadianItemLine>,
}

/// Complete Moadian E-Invoice Payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoadianInvoicePayload {
    pub header: MoadianHeader,
    pub body: Vec<MoadianItemLine>,
}

impl MoadianInvoicePayload {
    /// Constructs a Moadian e-invoice and computes all aggregated financial totals.
    pub fn new(params: MoadianInvoiceParams<'_>) -> Result<Self, MoadianError> {
        if params.items.is_empty() {
            return Err(MoadianError::EmptyInvoice);
        }

        let seller_tin: CompactString = params.seller_tax_id.into();
        if seller_tin.trim().is_empty() {
            return Err(MoadianError::MissingSellerTaxId);
        }

        let tprdis: Decimal = params.items.iter().map(|i| i.prdis).sum();
        let tdis: Decimal = params.items.iter().map(|i| i.dis).sum();
        let tadis: Decimal = params.items.iter().map(|i| i.adis).sum();
        let tvam: Decimal = params.items.iter().map(|i| i.vam).sum();
        let todam = Decimal::ZERO;
        let tbill = tadis + tvam + todam;

        let header = MoadianHeader {
            taxid: params.taxid.into(),
            indatim: params.posting_datetime.timestamp_millis(),
            inty: if params.buyer_national_id.is_some() {
                1
            } else {
                2
            },
            inno: params.inno.into(),
            inp: 1, // Sales
            ins: params.status as u32,
            tins: seller_tin,
            bid: params.buyer_national_id.map(Into::into),
            tinb: params.buyer_tax_id.map(Into::into),
            tprdis,
            tdis,
            tadis,
            tvam,
            todam,
            tbill,
        };

        Ok(Self {
            header,
            body: params.items,
        })
    }

    /// Produces a canonical JSON string with sorted keys at all levels and compact formatting.
    pub fn to_canonical_json(&self) -> Result<String, MoadianError> {
        let val = serde_json::to_value(self)
            .map_err(|e| MoadianError::SerializationError(e.to_string()))?;
        let canonical_val = Self::canonicalize_value(val);
        serde_json::to_string(&canonical_val)
            .map_err(|e| MoadianError::SerializationError(e.to_string()))
    }

    fn canonicalize_value(val: serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Object(map) => {
                let mut btree = BTreeMap::new();
                for (k, v) in map {
                    btree.insert(k, Self::canonicalize_value(v));
                }
                serde_json::Value::Object(btree.into_iter().collect())
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Self::canonicalize_value).collect())
            }
            other => other,
        }
    }

    /// Computes canonical SHA-256 payload digest for digital signing.
    #[must_use]
    pub fn compute_payload_hash(&self) -> String {
        let json_bytes = self
            .to_canonical_json()
            .unwrap_or_else(|_| serde_json::to_string(self).unwrap_or_default());
        let mut hasher = Sha256::new();
        hasher.update(json_bytes.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Signs the canonical payload using ECDSA P-256 / SHA-256 and formats the envelope for INTA API v2.
    pub fn sign_envelope(
        &self,
        fiscal_id: &str,
        signing_key: &SigningKey,
        encryption_key_id: Option<&str>,
    ) -> Result<MoadianSignedEnvelope, MoadianError> {
        let canonical_json = self.to_canonical_json()?;
        let signature_base64 = sign_canonical_payload(&canonical_json, signing_key)?;

        Ok(MoadianSignedEnvelope {
            packet_type: "INVOICE.V02".to_string(),
            fiscal_id: fiscal_id.to_string(),
            data: canonical_json,
            encryption_key_id: encryption_key_id.map(ToString::to_string),
            signature: signature_base64,
        })
    }
}

/// Signed Moadian E-Invoice packet formatted according to INTA API v2 specifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoadianSignedEnvelope {
    #[serde(rename = "packetType")]
    pub packet_type: String, // "INVOICE.V02"
    #[serde(rename = "fiscalId")]
    pub fiscal_id: String,
    pub data: String, // Canonical JSON payload
    #[serde(rename = "encryptionKeyId", skip_serializing_if = "Option::is_none")]
    pub encryption_key_id: Option<String>,
    pub signature: String, // Base64 DER ECDSA P-256 signature
}

/// Signs canonical UTF-8 JSON bytes using ECDSA P-256 and returns a Base64-encoded DER signature.
pub fn sign_canonical_payload(
    canonical_json: &str,
    signing_key: &SigningKey,
) -> Result<String, MoadianError> {
    let signature: Signature = signing_key.sign(canonical_json.as_bytes());
    let der_bytes = signature.to_der();
    Ok(BASE64_STANDARD.encode(der_bytes.as_bytes()))
}

/// Verifies a Base64-encoded DER ECDSA P-256 signature against canonical JSON payload.
pub fn verify_canonical_payload_signature(
    canonical_json: &str,
    signature_base64: &str,
    verifying_key: &VerifyingKey,
) -> Result<bool, MoadianError> {
    let der_bytes = BASE64_STANDARD
        .decode(signature_base64.trim())
        .map_err(|e| MoadianError::VerificationError(format!("Invalid Base64 signature: {e}")))?;
    let signature = Signature::from_der(&der_bytes)
        .map_err(|e| MoadianError::VerificationError(format!("Invalid DER signature: {e}")))?;
    Ok(verifying_key
        .verify(canonical_json.as_bytes(), &signature)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moadian_tax_id_generation_and_checksum() {
        let dt = DateTime::from_timestamp(1760616000, 0).unwrap(); // 2025-10-16
        let tax_id = MoadianTaxIdGenerator::generate_tax_id("A12345", dt, 1042).unwrap();

        assert_eq!(tax_id.len(), 22);
        assert!(tax_id.starts_with("A12345"));

        // Check digit verification
        let raw_21 = &tax_id[0..21];
        let expected_check = MoadianTaxIdGenerator::compute_check_digit(raw_21);
        assert_eq!(tax_id.chars().last().unwrap(), expected_check);
    }

    #[test]
    fn test_moadian_invoice_vat_totals_and_hash() {
        let dt = DateTime::from_timestamp(1760616000, 0).unwrap();
        let tax_id = MoadianTaxIdGenerator::generate_tax_id("B67890", dt, 1).unwrap();

        let items = vec![
            MoadianItemLine::new(
                "2710000000001",
                "Industrial PLC Controller",
                dec!(2.0),
                dec!(1000.0), // $2000 gross
                dec!(100.0),  // $100 discount -> $1900 net
                dec!(10.0),   // 10% VAT -> $190 VAT
            ),
            MoadianItemLine::new(
                "2710000000002",
                "Sensors Package",
                dec!(5.0),
                dec!(200.0), // $1000 gross
                Decimal::ZERO,
                dec!(10.0), // 10% VAT -> $100 VAT
            ),
        ];

        let invoice = MoadianInvoicePayload::new(MoadianInvoiceParams {
            taxid: &tax_id,
            inno: "INV-2026-001",
            seller_tax_id: "14001234567",
            buyer_national_id: Some("10101234567"),
            buyer_tax_id: Some("411112345678"),
            posting_datetime: dt,
            status: MoadianInvoiceStatus::Main,
            items,
        })
        .unwrap();

        assert_eq!(invoice.header.tprdis, dec!(3000.0)); // 2000 + 1000
        assert_eq!(invoice.header.tdis, dec!(100.0));
        assert_eq!(invoice.header.tadis, dec!(2900.0));
        assert_eq!(invoice.header.tvam, dec!(290.0)); // 190 + 100
        assert_eq!(invoice.header.tbill, dec!(3190.0)); // 2900 + 290

        let hash = invoice.compute_payload_hash();
        assert_eq!(hash.len(), 64); // SHA-256 hex string
    }

    #[test]
    fn test_moadian_canonical_json_and_ecdsa_p256_signing() {
        let dt = DateTime::from_timestamp(1760616000, 0).unwrap();
        let tax_id = MoadianTaxIdGenerator::generate_tax_id("A12345", dt, 99).unwrap();

        let items = vec![MoadianItemLine::new(
            "2710000000001",
            "High-Precision Thermal Sensor",
            dec!(1.0),
            dec!(500.0),
            Decimal::ZERO,
            dec!(10.0),
        )];

        let invoice = MoadianInvoicePayload::new(MoadianInvoiceParams {
            taxid: &tax_id,
            inno: "INV-2026-999",
            seller_tax_id: "14009999999",
            buyer_national_id: None,
            buyer_tax_id: None,
            posting_datetime: dt,
            status: MoadianInvoiceStatus::Main,
            items,
        })
        .unwrap();

        // 1. Verify canonical JSON serialization produces deterministic sorted keys
        let canonical_json = invoice.to_canonical_json().expect("Canonical JSON");
        assert!(canonical_json.starts_with(r#"{"body":[{"#));
        assert!(canonical_json.contains(r#""header":{"#));

        // 2. Instantiate a deterministic P-256 signing key
        let test_scalar = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let signing_key =
            SigningKey::from_slice(&test_scalar).expect("Deterministic valid P-256 scalar");
        let verifying_key = VerifyingKey::from(&signing_key);

        // 3. Sign invoice envelope
        let envelope = invoice
            .sign_envelope("A12345", &signing_key, Some("KEY-01"))
            .expect("Sign envelope");

        assert_eq!(envelope.packet_type, "INVOICE.V02");
        assert_eq!(envelope.fiscal_id, "A12345");
        assert_eq!(envelope.encryption_key_id.as_deref(), Some("KEY-01"));
        assert!(!envelope.signature.is_empty());

        // 4. Verify valid signature
        let is_valid =
            verify_canonical_payload_signature(&envelope.data, &envelope.signature, &verifying_key)
                .expect("Verify signature");
        assert!(is_valid, "Signature must verify successfully");

        // 5. Verify tampered payload fails verification
        let tampered_data = envelope.data.replace("INV-2026-999", "INV-2026-TAMPERED");
        let is_valid_tampered =
            verify_canonical_payload_signature(&tampered_data, &envelope.signature, &verifying_key)
                .expect("Verify signature");
        assert!(
            !is_valid_tampered,
            "Tampered payload must fail signature check"
        );
    }
}
