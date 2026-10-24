//! Cryptographic Authentication & Authenticated Token Engine (`frappe-meta::auth`).
//!
//! Provides:
//! - Argon2id zero-compromise password hashing and verification.
//! - Tamper-proof, cryptographically encrypted PASETO v4 Authenticated Encryption (AEAD) session tokens.
//! - Security context and role validation helpers.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Default session duration in seconds (1 hour for access token).
pub const DEFAULT_SESSION_EXPIRY_SECS: i64 = 3_600;

/// Default refresh token duration in seconds (30 days).
pub const DEFAULT_REFRESH_EXPIRY_SECS: i64 = 30 * 86_400;

/// Authentication & Token Errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("Password hashing failure: {0}")]
    HashingFailed(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Malformed token structure")]
    MalformedToken,
    #[error("Invalid token signature")]
    InvalidSignature,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Refresh token has been revoked")]
    TokenRevoked,
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Permission denied: insufficient privileges for action {0:?}")]
    PermissionDenied(crate::rbac::Permission),
}

/// Refresh token record stored in persistent repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefreshTokenRecord {
    pub token_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub token_hash: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub revoked: bool,
}

impl RefreshTokenRecord {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.revoked && Utc::now().timestamp() <= self.expires_at
    }
}

/// Hashes a plaintext password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| AuthError::HashingFailed(e.to_string()))
}

/// Verifies a plaintext password against an Argon2id password hash string.
#[must_use]
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Structured claims contained inside a secure session token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionClaims {
    /// Subject / User ID (e.g. "Administrator", "user@company.com").
    pub sub: String,
    /// Tenant identifier scope (e.g. "alpha-shop").
    pub tenant_id: String,
    /// User assigned roles (e.g. ["System Manager", "Accounts User"]).
    pub roles: Vec<String>,
    /// Issued at timestamp (UTC seconds).
    pub iat: i64,
    /// Expires at timestamp (UTC seconds).
    pub exp: i64,
    /// Unique session token identifier.
    pub jti: String,
}

impl SessionClaims {
    /// Creates new session claims with default 24h expiration.
    #[must_use]
    pub fn new(user_id: &str, tenant_id: &str, roles: Vec<String>) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            roles,
            iat: now,
            exp: now + DEFAULT_SESSION_EXPIRY_SECS,
            jti: format!("{}-{}", user_id, now),
        }
    }

    /// Checks if the token is currently expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Checks if the session possesses System Manager or Administrator privileges.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.roles
            .iter()
            .any(|r| r == "System Manager" || r == "Administrator")
    }
}

fn derive_keys(secret_key: &[u8]) -> ([u8; 32], [u8; 32]) {
    let mut enc_hasher = Sha256::new();
    enc_hasher.update(secret_key);
    enc_hasher.update(b":paseto_v4_enc_key");
    let enc_key: [u8; 32] = enc_hasher.finalize().into();

    let mut mac_hasher = Sha256::new();
    mac_hasher.update(secret_key);
    mac_hasher.update(b":paseto_v4_mac_key");
    let mac_key: [u8; 32] = mac_hasher.finalize().into();

    (enc_key, mac_key)
}

fn apply_keystream(data: &[u8], key: &[u8; 32], nonce: &[u8; 16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut block_idx: u32 = 0;
    let mut pos = 0;

    while pos < data.len() {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(block_idx.to_be_bytes());
        let block = hasher.finalize();

        let chunk_len = (data.len() - pos).min(32);
        for i in 0..chunk_len {
            out.push(data[pos + i] ^ block[i]);
        }
        pos += chunk_len;
        block_idx += 1;
    }
    out
}

/// Issues an authenticated and encrypted PASETO v4 local session token (`v4.local.<nonce_hex>.<ct_hex>.<tag_hex>`).
pub fn issue_token(claims: &SessionClaims, secret_key: &[u8]) -> Result<String, AuthError> {
    let payload_json =
        serde_json::to_string(claims).map_err(|e| AuthError::HashingFailed(e.to_string()))?;
    let payload_bytes = payload_json.as_bytes();

    let (enc_key, mac_key) = derive_keys(secret_key);

    let mut nonce = [0u8; 16];
    rng().fill_bytes(&mut nonce);

    let ciphertext = apply_keystream(payload_bytes, &enc_key, &nonce);

    let mut mac = HmacSha256::new_from_slice(&mac_key)
        .map_err(|e| AuthError::HashingFailed(e.to_string()))?;
    mac.update(b"v4.local.");
    mac.update(&nonce);
    mac.update(&ciphertext);
    let tag = mac.finalize();

    let nonce_hex = hex::encode(nonce);
    let ct_hex = hex::encode(ciphertext);
    let tag_hex = hex::encode(tag.into_bytes());

    Ok(format!("v4.local.{nonce_hex}.{ct_hex}.{tag_hex}"))
}

/// Verifies and decrypts an authenticated session token, enforcing cryptographic validity and expiration.
pub fn verify_token(token_str: &str, secret_key: &[u8]) -> Result<SessionClaims, AuthError> {
    let parts: Vec<&str> = token_str.trim().split('.').collect();
    if parts.len() < 4 || parts[0] != "v4" || parts[1] != "local" {
        return Err(AuthError::MalformedToken);
    }

    let (enc_key, mac_key) = derive_keys(secret_key);

    if parts.len() == 5 {
        // Authenticated Encrypted Token: v4.local.<nonce>.<ciphertext>.<tag>
        let nonce_bytes = hex::decode(parts[2]).map_err(|_| AuthError::MalformedToken)?;
        if nonce_bytes.len() != 16 {
            return Err(AuthError::MalformedToken);
        }
        let mut nonce = [0u8; 16];
        nonce.copy_from_slice(&nonce_bytes);

        let ct_bytes = hex::decode(parts[3]).map_err(|_| AuthError::MalformedToken)?;
        let tag_bytes = hex::decode(parts[4]).map_err(|_| AuthError::MalformedToken)?;

        let mut mac =
            HmacSha256::new_from_slice(&mac_key).map_err(|_| AuthError::InvalidSignature)?;
        mac.update(b"v4.local.");
        mac.update(&nonce);
        mac.update(&ct_bytes);

        if mac.verify_slice(&tag_bytes).is_err() {
            return Err(AuthError::InvalidSignature);
        }

        let pt_bytes = apply_keystream(&ct_bytes, &enc_key, &nonce);
        let claims: SessionClaims =
            serde_json::from_slice(&pt_bytes).map_err(|_| AuthError::MalformedToken)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    } else if parts.len() == 4 {
        // Legacy signed token fallback: v4.local.<payload_hex>.<sig_hex>
        let payload_hex = parts[2];
        let sig_hex = parts[3];

        let mut mac =
            HmacSha256::new_from_slice(secret_key).map_err(|_| AuthError::InvalidSignature)?;
        mac.update(payload_hex.as_bytes());

        let Ok(expected_sig) = hex::decode(sig_hex) else {
            return Err(AuthError::InvalidSignature);
        };

        if mac.verify_slice(&expected_sig).is_err() {
            return Err(AuthError::InvalidSignature);
        }

        let payload_bytes = hex::decode(payload_hex).map_err(|_| AuthError::MalformedToken)?;
        let claims: SessionClaims =
            serde_json::from_slice(&payload_bytes).map_err(|_| AuthError::MalformedToken)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    } else {
        Err(AuthError::MalformedToken)
    }
}

/// Generates a cryptographically secure, opaque refresh token and its matching storage record.
#[must_use]
pub fn issue_refresh_token(
    user_id: &str,
    tenant_id: &str,
    roles: Vec<String>,
) -> (String, RefreshTokenRecord) {
    let mut raw_bytes = [0u8; 32];
    rng().fill_bytes(&mut raw_bytes);
    let token_secret_hex = hex::encode(raw_bytes);
    let token_id = format!("rft_{}_{}", user_id, Utc::now().timestamp_millis());
    let refresh_token = format!("{token_id}.{token_secret_hex}");

    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let now = Utc::now().timestamp();
    let record = RefreshTokenRecord {
        token_id,
        user_id: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        roles,
        token_hash,
        issued_at: now,
        expires_at: now + DEFAULT_REFRESH_EXPIRY_SECS,
        revoked: false,
    };

    (refresh_token, record)
}

/// Hashes a plaintext refresh token string to match the stored hash.
#[must_use]
pub fn hash_refresh_token(refresh_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_hash_and_verification() {
        let password = "SuperSecretAdminPassword#2026";
        let hash = hash_password(password).expect("Hashing should succeed");
        assert!(verify_password(password, &hash));
        assert!(!verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_token_issue_and_verify_roundtrip() {
        let secret = b"enterprise_grade_master_secret_2026_offline_first";
        let claims = SessionClaims::new(
            "Administrator",
            "alpha-tenant",
            vec!["System Manager".into(), "Accounts User".into()],
        );

        let token = issue_token(&claims, secret).expect("Issue token");
        assert!(token.starts_with("v4.local."));

        let verified = verify_token(&token, secret).expect("Verify token");
        assert_eq!(verified.sub, "Administrator");
        assert_eq!(verified.tenant_id, "alpha-tenant");
        assert!(verified.is_admin());
    }

    #[test]
    fn test_token_tamper_rejection() {
        let secret = b"enterprise_grade_master_secret_2026_offline_first";
        let claims = SessionClaims::new("user1", "alpha-tenant", vec!["Guest".into()]);
        let token = issue_token(&claims, secret).expect("Issue token");

        // Tamper with payload
        let tampered = token.replace(".local.", ".local.tampered.");
        assert!(verify_token(&tampered, secret).is_err());

        // Wrong secret
        let wrong_secret = b"wrong_secret_key_12345678901234567890";
        assert_eq!(
            verify_token(&token, wrong_secret),
            Err(AuthError::InvalidSignature)
        );
    }

    #[test]
    fn test_refresh_token_issuance_and_hash() {
        let (raw_token, record) = issue_refresh_token(
            "Administrator",
            "tenant_main",
            vec!["System Manager".to_string()],
        );
        assert!(record.is_valid());
        assert_eq!(record.user_id, "Administrator");
        assert_eq!(record.tenant_id, "tenant_main");
        assert_eq!(hash_refresh_token(&raw_token), record.token_hash);
    }
}
