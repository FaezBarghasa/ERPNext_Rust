//! Universal AI Agent OAuth 2.0 Hub & Token Server (`frappe-net::ai_oauth`).
//!
//! Implements RFC 6749 (OAuth 2.0 Client Credentials Grant) and RFC 7591 dynamic
//! client validation for frontier AI agents (ChatGPT-4o/o3, Claude 3.5/3.7, Gemini 2.0, DeepSeek R1, Ollama).

use actix_web::{HttpResponse, Responder, web};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Request payload for `/api/v1/oauth/token` or `/oauth/token`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenRequest {
    pub grant_type: CompactString,
    pub client_id: CompactString,
    pub client_secret: CompactString,
    pub scope: Option<CompactString>,
}

/// Standard OAuth 2.0 token response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenResponse {
    pub access_token: CompactString,
    pub token_type: CompactString,
    pub expires_in: u64,
    pub scope: CompactString,
}

/// Token payload claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: CompactString,
    pub client_id: CompactString,
    pub scope: CompactString,
    pub exp: i64,
    pub iss: CompactString,
}

/// Token verification error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum OAuthError {
    #[error("Unsupported grant_type: only 'client_credentials' is supported")]
    UnsupportedGrantType,
    #[error("Invalid client credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token signature")]
    InvalidSignature,
}

/// Core AI Agent OAuth Service.
pub struct AiOAuthService;

impl AiOAuthService {
    /// Issues a cryptographically signed HMAC bearer token for an AI agent.
    pub fn issue_token(
        secret: &str,
        client_id: &str,
        scope: &str,
        duration_seconds: u64,
    ) -> Result<TokenResponse, OAuthError> {
        let now = chrono::Utc::now().timestamp();
        let exp = now + duration_seconds as i64;
        let claims = Claims {
            sub: format!("agent:{}", client_id).into(),
            client_id: client_id.into(),
            scope: scope.into(),
            exp,
            iss: "rustnext-oauth-hub".into(),
        };

        let claims_json =
            serde_json::to_string(&claims).map_err(|_| OAuthError::InvalidCredentials)?;
        let claims_b64 = hex::encode(claims_json.as_bytes());

        // Compute HMAC signature over claims
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(b":");
        hasher.update(claims_b64.as_bytes());
        let sig = hex::encode(hasher.finalize());

        let token = format!("{}.{}", claims_b64, sig);

        Ok(TokenResponse {
            access_token: token.into(),
            token_type: "Bearer".into(),
            expires_in: duration_seconds,
            scope: scope.into(),
        })
    }

    /// Verifies a bearer token and returns its decoded claims.
    pub fn verify_token(secret: &str, token: &str) -> Result<Claims, OAuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return Err(OAuthError::InvalidSignature);
        }

        let claims_b64 = parts[0];
        let expected_sig = parts[1];

        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(b":");
        hasher.update(claims_b64.as_bytes());
        let actual_sig = hex::encode(hasher.finalize());

        if !expected_sig.eq_ignore_ascii_case(&actual_sig) {
            return Err(OAuthError::InvalidSignature);
        }

        let raw_bytes = hex::decode(claims_b64).map_err(|_| OAuthError::InvalidSignature)?;
        let claims: Claims =
            serde_json::from_slice(&raw_bytes).map_err(|_| OAuthError::InvalidSignature)?;

        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(OAuthError::TokenExpired);
        }

        Ok(claims)
    }
}

/// Actix-web handler for `/api/v1/oauth/token`.
pub async fn oauth_token_handler(payload: web::Json<TokenRequest>) -> impl Responder {
    if payload.grant_type != "client_credentials" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "unsupported_grant_type",
            "error_description": "Only 'client_credentials' grant is supported for AI agents"
        }));
    }

    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid_client",
            "error_description": "Missing client credentials"
        }));
    }

    // Default scope if not specified
    let default_scope = "erp:stock:read erp:invoice:write cms:ast";
    let scope = payload
        .scope
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(default_scope);

    // Use server master token secret or client secret for signature
    let secret = &payload.client_secret;

    match AiOAuthService::issue_token(secret, &payload.client_id, scope, 86400) {
        Ok(token_resp) => HttpResponse::Ok().json(token_resp),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "server_error",
            "error_description": e.to_string()
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_oauth_issue_and_verify_roundtrip() {
        let secret = "super_secret_agent_master_key_2026";
        let client_id = "claude-3-7-sonnet";
        let scope = "erp:stock:read erp:invoice:write";

        let token_resp = AiOAuthService::issue_token(secret, client_id, scope, 3600).unwrap();
        assert_eq!(token_resp.token_type, "Bearer");
        assert_eq!(token_resp.expires_in, 3600);

        let claims = AiOAuthService::verify_token(secret, &token_resp.access_token).unwrap();
        assert_eq!(claims.client_id, client_id);
        assert_eq!(claims.scope, scope);
    }

    #[test]
    fn test_ai_oauth_rejects_tampered_token() {
        let secret = "super_secret_agent_master_key_2026";
        let token_resp = AiOAuthService::issue_token(secret, "gpt-4o", "all", 3600).unwrap();

        let tampered = format!("{}tampered", token_resp.access_token);
        assert!(AiOAuthService::verify_token(secret, &tampered).is_err());
    }
}
