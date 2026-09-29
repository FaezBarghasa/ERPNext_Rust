//! Multi-Factor Authentication (MFA / TOTP) Engine (`frappe-meta::mfa`).
//!
//! Provides RFC 6238 / RFC 4226 Time-Based One-Time Password (TOTP) generation,
//! verification with time-drift skew tolerance, Base32 RFC 4648 encoding/decoding,
//! single-use recovery backup codes, and replay attack prevention.

use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

/// Base32 standard alphabet (RFC 4648).
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// MFA & TOTP Engine errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MfaError {
    #[error("Invalid Base32 secret string: {0}")]
    InvalidBase32(String),
    #[error("Invalid TOTP code format: expected {expected} digits")]
    InvalidCodeFormat { expected: u32 },
    #[error("TOTP code verification failed")]
    VerificationFailed,
    #[error("TOTP code already used in this time window (replay detected)")]
    ReplayDetected,
    #[error("Invalid or exhausted backup recovery code")]
    InvalidBackupCode,
    #[error("Cryptographic HMAC failure: {0}")]
    HmacFailure(String),
}

/// Base32 encoding implementation (RFC 4648).
#[must_use]
pub fn base32_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len().div_ceil(5) * 8);
    let mut buffer: u64 = 0;
    let mut bits_left: u32 = 0;

    for &byte in data {
        buffer = (buffer << 8) | u64::from(byte);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(BASE32_ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(BASE32_ALPHABET[index] as char);
    }

    result
}

/// Base32 decoding implementation (RFC 4648).
pub fn base32_decode(input: &str) -> Result<Vec<u8>, MfaError> {
    let cleaned = input.trim().replace([' ', '-'], "").to_ascii_uppercase();
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity(cleaned.len() * 5 / 8);
    let mut buffer: u64 = 0;
    let mut bits_left: u32 = 0;

    for ch in cleaned.chars() {
        if ch == '=' {
            break;
        }
        let val = match ch {
            'A'..='Z' => (ch as u8) - b'A',
            '2'..='7' => (ch as u8) - b'2' + 26,
            _ => return Err(MfaError::InvalidBase32(format!("Invalid character: {ch}"))),
        };

        buffer = (buffer << 5) | u64::from(val);
        bits_left += 5;

        if bits_left >= 8 {
            bits_left -= 8;
            result.push(((buffer >> bits_left) & 0xFF) as u8);
        }
    }

    Ok(result)
}

/// TOTP Configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TotpConfig {
    /// Number of digits in generated TOTP token (default 6).
    pub digits: u32,
    /// Time step in seconds (default 30).
    pub step_secs: u64,
    /// Number of steps allowed before/after current step for clock drift (default 1).
    pub skew_steps: i64,
    /// Issuer organization name (e.g. "RustNext ERP").
    pub issuer: String,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            digits: 6,
            step_secs: 30,
            skew_steps: 1,
            issuer: "RustNext ERP".to_string(),
        }
    }
}

/// Generates a cryptographically secure Base32 secret key (default 20 bytes / 160 bits).
#[must_use]
pub fn generate_totp_secret(length_bytes: usize) -> (String, Vec<u8>) {
    let mut raw = vec![0u8; length_bytes.max(20)];
    rng().fill_bytes(&mut raw);
    let encoded = base32_encode(&raw);
    (encoded, raw)
}

/// Generates a set of 8-character alphanumeric backup recovery codes (e.g., `ABCD-1234`).
#[must_use]
pub fn generate_backup_codes(count: usize) -> Vec<String> {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    (0..count)
        .map(|_| {
            let mut code = String::with_capacity(9);
            for i in 0..8 {
                if i == 4 {
                    code.push('-');
                }
                let idx = (rand::random::<u32>() as usize) % CHARSET.len();
                code.push(CHARSET[idx] as char);
            }
            code
        })
        .collect()
}

/// Generates an RFC 6238 TOTP token for a specific UNIX timestamp.
pub fn compute_totp(
    secret_bytes: &[u8],
    timestamp_secs: u64,
    config: &TotpConfig,
) -> Result<String, MfaError> {
    let counter = timestamp_secs / config.step_secs;
    let counter_bytes = counter.to_be_bytes();

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret_bytes)
        .map_err(|e| MfaError::HmacFailure(e.to_string()))?;
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0F) as usize;
    let binary_code = ((u32::from(result[offset] & 0x7F)) << 24)
        | ((u32::from(result[offset + 1])) << 16)
        | ((u32::from(result[offset + 2])) << 8)
        | (u32::from(result[offset + 3]));

    let modulo = 10u32.pow(config.digits);
    let token_val = binary_code % modulo;

    Ok(format!(
        "{:0>width$}",
        token_val,
        width = config.digits as usize
    ))
}

/// Generates an `otpauth://` URI for rendering TOTP QR codes in authenticator apps.
#[must_use]
pub fn generate_otpauth_uri(
    secret_base32: &str,
    account_name: &str,
    issuer: &str,
    config: &TotpConfig,
) -> String {
    let clean_issuer = issuer.trim().replace(':', "");
    let clean_account = account_name.trim().replace(':', "");
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA256&digits={}&period={}",
        clean_issuer, clean_account, secret_base32, clean_issuer, config.digits, config.step_secs
    )
}

/// Multi-Factor Authentication State Record for a User.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MfaRecord {
    pub user_id: String,
    pub secret: String,
    pub enabled: bool,
    pub backup_codes: Vec<String>,
    pub last_used_step: u64,
    pub created_at: String,
    pub enabled_at: Option<String>,
}

impl MfaRecord {
    /// Initializes an unconfirmed MFA enrollment with newly generated secret and 10 backup codes.
    #[must_use]
    pub fn new_enrollment(user_id: &str) -> (Self, Vec<String>) {
        let (secret, _) = generate_totp_secret(20);
        let backup_codes = generate_backup_codes(10);
        let record = Self {
            user_id: user_id.to_string(),
            secret,
            enabled: false,
            backup_codes: backup_codes.clone(),
            last_used_step: 0,
            created_at: Utc::now().to_rfc3339(),
            enabled_at: None,
        };
        (record, backup_codes)
    }

    /// Verifies and activates MFA with an initial valid token.
    pub fn activate(
        &mut self,
        token: &str,
        timestamp_secs: u64,
        config: &TotpConfig,
    ) -> Result<(), MfaError> {
        let secret_bytes = base32_decode(&self.secret)?;
        let step = timestamp_secs / config.step_secs;

        // Verify across skew window
        let mut matched = false;

        for offset in -config.skew_steps..=config.skew_steps {
            let target_step = (step as i64 + offset).max(0) as u64;
            let target_time = target_step * config.step_secs;
            if compute_totp(&secret_bytes, target_time, config)
                .is_ok_and(|expected| expected == token.trim())
            {
                matched = true;
                break;
            }
        }

        if !matched {
            return Err(MfaError::VerificationFailed);
        }

        self.enabled = true;
        self.last_used_step = 0;
        self.enabled_at = Some(Utc::now().to_rfc3339());
        Ok(())
    }

    /// Verifies a login TOTP token or single-use backup recovery code.
    pub fn verify(
        &mut self,
        code_or_backup: &str,
        timestamp_secs: u64,
        config: &TotpConfig,
    ) -> Result<bool, MfaError> {
        if !self.enabled {
            return Ok(true); // MFA not enforced
        }

        let clean = code_or_backup.trim().replace('-', "").to_ascii_uppercase();

        // 1. Check if it's a backup recovery code
        for (i, backup) in self.backup_codes.iter().enumerate() {
            let clean_backup = backup.replace('-', "").to_ascii_uppercase();
            if clean_backup == clean {
                // Consume single-use backup code
                self.backup_codes.remove(i);
                return Ok(true);
            }
        }

        // 2. Check TOTP Code
        let secret_bytes = base32_decode(&self.secret)?;
        let step = timestamp_secs / config.step_secs;

        for offset in -config.skew_steps..=config.skew_steps {
            let target_step = (step as i64 + offset).max(0) as u64;
            if target_step <= self.last_used_step && offset == 0 {
                // Potential replay
                continue;
            }

            let target_time = target_step * config.step_secs;
            if compute_totp(&secret_bytes, target_time, config)
                .is_ok_and(|expected| expected == clean)
            {
                self.last_used_step = target_step;
                return Ok(true);
            }
        }

        Err(MfaError::VerificationFailed)
    }

    /// Disables MFA for the user.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.enabled_at = None;
        self.backup_codes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_roundtrip() {
        let input = b"Hello, Enterprise Rust World!";
        let encoded = base32_encode(input);
        let decoded = base32_decode(&encoded).expect("base32 decode");
        assert_eq!(input.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_totp_generation_and_verification() {
        let (record, _) = MfaRecord::new_enrollment("usr_admin");
        let config = TotpConfig::default();
        let timestamp = 1_700_000_000u64;

        let secret_bytes = base32_decode(&record.secret).expect("secret decode");
        let token = compute_totp(&secret_bytes, timestamp, &config).expect("compute totp");
        assert_eq!(token.len(), 6);

        let mut active_record = record.clone();
        assert!(active_record.activate(&token, timestamp, &config).is_ok());
        assert!(active_record.enabled);

        // Verification on same step succeeds
        assert!(active_record.verify(&token, timestamp + 5, &config).is_ok());

        // Verification in +30s skew window succeeds
        let next_step_token =
            compute_totp(&secret_bytes, timestamp + 30, &config).expect("next step totp");
        assert!(
            active_record
                .verify(&next_step_token, timestamp + 30, &config)
                .is_ok()
        );

        // Wrong token fails
        assert!(
            active_record
                .verify("000000", timestamp + 30, &config)
                .is_err()
        );
    }

    #[test]
    fn test_backup_recovery_code_consumption() {
        let (mut record, backup_codes) = MfaRecord::new_enrollment("usr_tester");
        let config = TotpConfig::default();
        let timestamp = 1_700_000_000u64;

        let secret_bytes = base32_decode(&record.secret).unwrap();
        let token = compute_totp(&secret_bytes, timestamp, &config).unwrap();
        record.activate(&token, timestamp, &config).unwrap();

        assert_eq!(record.backup_codes.len(), 10);
        let first_code = backup_codes[0].clone();

        // Use backup code
        assert!(record.verify(&first_code, timestamp + 60, &config).unwrap());
        assert_eq!(record.backup_codes.len(), 9);

        // Reusing same backup code fails
        assert!(
            record
                .verify(&first_code, timestamp + 120, &config)
                .is_err()
        );
    }

    #[test]
    fn test_otpauth_uri() {
        let config = TotpConfig::default();
        let uri = generate_otpauth_uri(
            "JBSWY3DPEHPK3PXP",
            "admin@example.com",
            "RustNext ERP",
            &config,
        );
        assert!(uri.starts_with("otpauth://totp/RustNext ERP:admin@example.com"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=RustNext ERP"));
    }
}
