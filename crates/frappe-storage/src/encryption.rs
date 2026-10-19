//! Field-Level Envelope Encryption (AEAD) (`frappe-storage::encryption`).
//!
//! Implements authenticated envelope encryption using standard ChaCha20-Poly1305 (RFC 8439),
//! HKDF-HMAC-SHA256 tenant key derivation from a 256-bit Master Key Encryption Key (KEK),
//! and random 96-bit nonces per field encryption operation.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use hmac::{Hmac, Mac};
use rand::RngExt;
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Errors during envelope encryption or decryption.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncryptionError {
    #[error("Decryption failed: invalid ciphertext or MAC tag mismatch")]
    DecryptionFailed,
    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),
}

/// Field-Level Envelope Encryption Manager using ChaCha20-Poly1305 AEAD.
#[derive(Clone, Debug)]
pub struct EnvelopeEncryption {
    master_kek: [u8; 32],
}

impl EnvelopeEncryption {
    /// Creates a new envelope encryption manager with a 256-bit Master KEK.
    #[must_use]
    pub fn new(master_kek: [u8; 32]) -> Self {
        Self { master_kek }
    }

    /// Derives a deterministic 256-bit tenant Data Encryption Key (DEK) using HMAC-SHA256.
    #[must_use]
    pub fn derive_tenant_dek(&self, tenant_id: &str) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(&self.master_kek).expect("HMAC can accept key of any size");
        mac.update(b"tenant-dek:");
        mac.update(tenant_id.as_bytes());
        mac.finalize().into_bytes().into()
    }

    /// Encrypts sensitive plaintext using the tenant's DEK with Authenticated Additional Data (AAD).
    ///
    /// Output payload format: `[12-byte Nonce] || [Ciphertext] || [16-byte Poly1305 Tag]`.
    #[must_use]
    pub fn encrypt_field(
        &self,
        tenant_id: &str,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Vec<u8> {
        let dek = self.derive_tenant_dek(tenant_id);
        let cipher = ChaCha20Poly1305::new(&Key::from(dek));
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data,
        };

        let ciphertext_with_tag = cipher
            .encrypt(&nonce, payload)
            .expect("ChaCha20-Poly1305 encryption cannot fail for in-memory buffer");

        let mut output = Vec::with_capacity(12 + ciphertext_with_tag.len());
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&ciphertext_with_tag);
        output
    }

    /// Decrypts ciphertext and validates authenticity tag against the provided AAD.
    pub fn decrypt_field(
        &self,
        tenant_id: &str,
        payload: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        // Minimum payload size: 12-byte Nonce + 16-byte Poly1305 Tag = 28 bytes
        if payload.len() < 28 {
            return Err(EncryptionError::DecryptionFailed);
        }

        let (nonce_bytes, ciphertext_with_tag) = payload.split_at(12);
        let dek = self.derive_tenant_dek(tenant_id);
        let cipher = ChaCha20Poly1305::new(&Key::from(dek));
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        let nonce = Nonce::from(nonce_array);

        let payload = Payload {
            msg: ciphertext_with_tag,
            aad: associated_data,
        };

        cipher
            .decrypt(&nonce, payload)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_encryption_roundtrip() {
        let master_kek = [0x42u8; 32];
        let encryption = EnvelopeEncryption::new(master_kek);

        let tenant = "tenant_clinic_berlin";
        let salary_data = b"$145,000.00 / annum | IBAN: DE89370400440532013000";
        let aad = b"tab_employee:EMP-00921/salary";

        let encrypted = encryption.encrypt_field(tenant, salary_data, aad);
        assert_ne!(encrypted.as_slice(), salary_data);

        let decrypted = encryption
            .decrypt_field(tenant, &encrypted, aad)
            .expect("Decryption must succeed with valid tag and AAD");
        assert_eq!(decrypted, salary_data);

        // Tampering detection on ciphertext
        let mut tampered = encrypted.clone();
        tampered[20] ^= 0xFF;
        assert_eq!(
            encryption.decrypt_field(tenant, &tampered, aad),
            Err(EncryptionError::DecryptionFailed)
        );

        // Cross-tenant isolation verification
        assert_eq!(
            encryption.decrypt_field("tenant_other", &encrypted, aad),
            Err(EncryptionError::DecryptionFailed)
        );

        // AAD mismatch verification
        let invalid_aad = b"tab_employee:EMP-00921/other_field";
        assert_eq!(
            encryption.decrypt_field(tenant, &encrypted, invalid_aad),
            Err(EncryptionError::DecryptionFailed)
        );
    }

    #[test]
    fn test_randomized_nonces_produce_distinct_ciphertexts() {
        let master_kek = [0x99u8; 32];
        let encryption = EnvelopeEncryption::new(master_kek);

        let tenant = "tenant_finance";
        let secret = b"super-confidential-token";
        let aad = b"context";

        let c1 = encryption.encrypt_field(tenant, secret, aad);
        let c2 = encryption.encrypt_field(tenant, secret, aad);

        // Nonces must be random, so ciphertexts must differ
        assert_ne!(c1, c2);

        // Both decrypt to identical plaintext
        assert_eq!(encryption.decrypt_field(tenant, &c1, aad).unwrap(), secret);
        assert_eq!(encryption.decrypt_field(tenant, &c2, aad).unwrap(), secret);
    }
}
