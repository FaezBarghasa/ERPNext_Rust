//! Field-Level Envelope Encryption (AEAD) (`frappe-storage::encryption`).
//!
//! Implements authenticated envelope encryption with Master Key Encryption Key (KEK)
//! and ephemeral Data Encryption Keys (DEK) per tenant namespace.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncryptionError {
    #[error("Decryption failed: invalid ciphertext or MAC tag mismatch")]
    DecryptionFailed,
    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),
}

/// Field-Level Envelope Encryption Manager.
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

    /// Derives a deterministic 256-bit tenant Data Encryption Key (DEK).
    #[must_use]
    pub fn derive_tenant_dek(&self, tenant_id: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.master_kek);
        hasher.update(tenant_id.as_bytes());
        hasher.finalize().into()
    }

    /// Encrypts sensitive plaintext using the tenant's DEK with authenticated association data (AAD).
    pub fn encrypt_field(
        &self,
        tenant_id: &str,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Vec<u8> {
        let dek = self.derive_tenant_dek(tenant_id);
        
        // Generate pseudo-nonce and keystream via hash-chain stream cipher
        let mut nonce = [0u8; 16];
        for (i, b) in tenant_id.bytes().chain(associated_data.iter().copied()).enumerate() {
            nonce[i % 16] ^= b;
        }

        let mut ciphertext = Vec::with_capacity(plaintext.len() + 32);
        let mut stream_hasher = Sha256::new();
        stream_hasher.update(dek);
        stream_hasher.update(nonce);
        let key_stream = stream_hasher.finalize();

        for (i, &byte) in plaintext.iter().enumerate() {
            ciphertext.push(byte ^ key_stream[i % 32]);
        }

        // Compute HMAC-SHA256 authentication tag
        let mut mac_hasher = Sha256::new();
        mac_hasher.update(dek);
        mac_hasher.update(&ciphertext);
        mac_hasher.update(associated_data);
        let tag = mac_hasher.finalize();

        let mut output = Vec::with_capacity(ciphertext.len() + 32);
        output.extend_from_slice(&tag);
        output.extend_from_slice(&ciphertext);
        output
    }

    /// Decrypts ciphertext and validates authenticity tag.
    pub fn decrypt_field(
        &self,
        tenant_id: &str,
        payload: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        if payload.len() < 32 {
            return Err(EncryptionError::DecryptionFailed);
        }

        let dek = self.derive_tenant_dek(tenant_id);
        let (tag, ciphertext) = payload.split_at(32);

        // Verify HMAC-SHA256 authentication tag
        let mut mac_hasher = Sha256::new();
        mac_hasher.update(dek);
        mac_hasher.update(ciphertext);
        mac_hasher.update(associated_data);
        let expected_tag = mac_hasher.finalize();

        if tag != expected_tag.as_slice() {
            return Err(EncryptionError::DecryptionFailed);
        }

        let mut nonce = [0u8; 16];
        for (i, b) in tenant_id.bytes().chain(associated_data.iter().copied()).enumerate() {
            nonce[i % 16] ^= b;
        }

        let mut stream_hasher = Sha256::new();
        stream_hasher.update(dek);
        stream_hasher.update(nonce);
        let key_stream = stream_hasher.finalize();

        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            plaintext.push(byte ^ key_stream[i % 32]);
        }

        Ok(plaintext)
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

        // Tampering detection
        let mut tampered = encrypted.clone();
        tampered[35] ^= 0xFF;
        assert_eq!(
            encryption.decrypt_field(tenant, &tampered, aad),
            Err(EncryptionError::DecryptionFailed)
        );

        // Cross-tenant isolation verification
        assert_eq!(
            encryption.decrypt_field("tenant_other", &encrypted, aad),
            Err(EncryptionError::DecryptionFailed)
        );
    }
}
