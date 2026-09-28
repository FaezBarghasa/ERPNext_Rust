//! Authentication & RBAC Guard Middleware (`frappe-net::middleware::auth`).
//!
//! Provides:
//! - Extracting and cryptographically verifying Bearer PASETO session tokens from requests.
//! - Injecting `SecurityContext` into request extensions.
//! - Role-Based Access Control (RBAC) verification at the HTTP edge.

use actix_web::{
    Error as ActixError, FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError,
    body::BoxBody,
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    http::{StatusCode, header},
};
use frappe_meta::auth::{AuthError, SessionClaims, verify_token};
use frappe_meta::rbac::{HasPermissionEdge, Permission, check_permission};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Default master token signing secret key for tenant cluster.
pub const MASTER_JWT_SECRET: &[u8] = b"rustnext_enterprise_paseto_master_secret_key_2026_offline_first";

/// Authenticated Security Context extracted from verified session tokens.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityContext {
    pub claims: SessionClaims,
}

impl SecurityContext {
    /// Creates a new security context from verified claims.
    #[must_use]
    pub fn new(claims: SessionClaims) -> Self {
        Self { claims }
    }

    /// Creates an administrative security context (e.g. for internal background jobs).
    #[must_use]
    pub fn root(tenant_id: &str) -> Self {
        Self {
            claims: SessionClaims::new(
                "Administrator",
                tenant_id,
                vec!["System Manager".into(), "Administrator".into()],
            ),
        }
    }

    /// Evaluates whether the security context has permission for a specific operation.
    pub fn check_perm(
        &self,
        permission_edges: &[HasPermissionEdge],
        perm: Permission,
        permlevel: u8,
    ) -> bool {
        check_permission(&self.claims.roles, permission_edges, perm, permlevel)
    }

    /// Enforces that the security context has permission for a specific operation, returning an error if denied.
    pub fn require_perm(
        &self,
        permission_edges: &[HasPermissionEdge],
        perm: Permission,
        permlevel: u8,
    ) -> Result<(), AuthError> {
        if self.check_perm(permission_edges, perm, permlevel) {
            Ok(())
        } else {
            Err(AuthError::PermissionDenied(perm))
        }
    }
}

impl FromRequest for SecurityContext {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(ctx) = req.extensions().get::<SecurityContext>() {
            return futures_util::future::ready(Ok(ctx.clone()));
        }

        // Extract from Authorization header: Bearer <token>
        if let Some(auth_hdr) = req.headers().get(header::AUTHORIZATION)
            && let Ok(auth_str) = auth_hdr.to_str()
        {
            let token_str = if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                stripped.trim()
            } else {
                auth_str.trim()
            };

            match verify_token(token_str, MASTER_JWT_SECRET) {
                Ok(claims) => {
                    let ctx = SecurityContext::new(claims);
                    return futures_util::future::ready(Ok(ctx));
                }
                Err(e) => return futures_util::future::ready(Err(ActixError::from(AuthMiddlewareError::InvalidToken(e.to_string())))),
            }
        }

        futures_util::future::ready(Err(ActixError::from(AuthMiddlewareError::MissingAuthorization)))
    }
}

/// HTTP Middleware errors for authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthMiddlewareError {
    #[error("Missing Authorization header")]
    MissingAuthorization,
    #[error("Invalid or expired session token: {0}")]
    InvalidToken(String),
    #[error("Permission denied: {0}")]
    Forbidden(String),
}

impl ResponseError for AuthMiddlewareError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingAuthorization | Self::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let body = serde_json::json!({
            "type": "https://datatracker.ietf.org/doc/html/rfc7807",
            "title": status.canonical_reason().unwrap_or("Forbidden"),
            "status": status.as_u16(),
            "detail": self.to_string(),
        });
        HttpResponse::build(status).json(body)
    }
}

/// Actix Web Middleware that enforces cryptographic token authentication on routes.
pub struct AuthGuard;

impl<S, B> Transform<S, ServiceRequest> for AuthGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type InitError = ();
    type Transform = AuthGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthGuardMiddleware {
            service: Arc::new(service),
        })
    }
}

pub struct AuthGuardMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();

        let token_opt = if let Some(auth_hdr) = headers.get(header::AUTHORIZATION)
            && let Ok(auth_str) = auth_hdr.to_str()
        {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                Some(stripped.trim().to_string())
            } else {
                Some(auth_str.trim().to_string())
            }
        } else {
            None
        };

        match token_opt {
            Some(token_str) => match verify_token(&token_str, MASTER_JWT_SECRET) {
                Ok(claims) => {
                    let ctx = SecurityContext::new(claims);
                    req.extensions_mut().insert(ctx);
                    let svc = self.service.clone();
                    Box::pin(async move { svc.call(req).await })
                }
                Err(e) => Box::pin(async move {
                    Err(ActixError::from(AuthMiddlewareError::InvalidToken(e.to_string())))
                }),
            },
            None => Box::pin(async move {
                Err(ActixError::from(AuthMiddlewareError::MissingAuthorization))
            }),
        }
    }
}
