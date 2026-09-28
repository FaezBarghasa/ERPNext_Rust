//! Cryptographic Authentication & Token Engine (`frappe-meta::auth`).
//!
//! Provides:
//! - Argon2id zero-compromise password hashing and verification.
//! - Tamper-proof, cryptographically signed PASETO-style session tokens with expiration and tenant scoping.
//! - Security context and role validation helpers.

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Default session duration in seconds (24 hours).
pub const DEFAULT_SESSION_EXPIRY_SECS: i64 = 86_400;

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
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Permission denied: insufficient privileges for action {0:?}")]
    PermissionDenied(crate::rbac::Permission),
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

/// Issues a cryptographically signed PASETO-style token (`v4.local.<payload_hex>.<sig_hex>`).
pub fn issue_token(claims: &SessionClaims, secret_key: &[u8]) -> Result<String, AuthError> {
    let payload_json =
        serde_json::to_string(claims).map_err(|e| AuthError::HashingFailed(e.to_string()))?;
    let payload_hex = hex::encode(payload_json.as_bytes());

    let mut mac = HmacSha256::new_from_slice(secret_key)
        .map_err(|e| AuthError::HashingFailed(e.to_string()))?;
    mac.update(payload_hex.as_bytes());
    let sig = mac.finalize();
    let sig_hex = hex::encode(sig.into_bytes());

    Ok(format!("v4.local.{payload_hex}.{sig_hex}"))
}

/// Verifies and decodes a signed session token, enforcing cryptographic validity and expiration.
pub fn verify_token(token_str: &str, secret_key: &[u8]) -> Result<SessionClaims, AuthError> {
    let parts: Vec<&str> = token_str.trim().split('.').collect();
    if parts.len() != 4 || parts[0] != "v4" || parts[1] != "local" {
        return Err(AuthError::MalformedToken);
    }

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
        let tampered = token.replace(".local.", ".local.tampered");
        assert_eq!(
            verify_token(&tampered, secret),
            Err(AuthError::InvalidSignature)
        );

        // Wrong secret
        let wrong_secret = b"wrong_secret_key_12345678901234567890";
        assert_eq!(
            verify_token(&token, wrong_secret),
            Err(AuthError::InvalidSignature)
        );
    }
}
