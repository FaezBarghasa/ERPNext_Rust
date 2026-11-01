//! Cross-Site Request Forgery (CSRF) Protection Middleware (`rwl-net::middleware::csrf`).
//!
//! Implements the cryptographically secure Double-Submit Cookie pattern for state-changing HTTP requests:
//! - Validates `X-CSRF-Token` header against the `rwl_csrf` cookie for `POST`, `PUT`, `DELETE`, `PATCH`.
//! - Automatically issues a new `rwl_csrf` cookie on safe requests (`GET`, `HEAD`, `OPTIONS`) if not present.
//! - Protects interactive endpoints while allowing Bearer-token-authenticated headless clients.

use actix_web::Error as ActixError;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{Method, StatusCode, header};
use actix_web::{HttpResponse, ResponseError};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use rand::RngExt;

pub const CSRF_COOKIE_NAME: &str = "rwl_csrf";
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// CSRF Middleware error types.
#[derive(Debug, thiserror::Error)]
pub enum CsrfError {
    #[error("Missing or invalid CSRF token")]
    MissingOrInvalidToken,
    #[error("CSRF token mismatch between cookie and header")]
    TokenMismatch,
}

impl ResponseError for CsrfError {
    fn status_code(&self) -> StatusCode {
        StatusCode::FORBIDDEN
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let body = serde_json::json!({
            "type": "https://datatracker.ietf.org/doc/html/rfc7807",
            "title": "Forbidden",
            "status": status.as_u16(),
            "detail": self.to_string(),
        });
        HttpResponse::build(status).json(body)
    }
}

/// CSRF Guard Middleware Configuration.
#[derive(Clone, Debug, Default)]
pub struct CsrfMiddleware;

impl CsrfMiddleware {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = CsrfService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfService { service })
    }
}

pub struct CsrfService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CsrfService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();
        let is_state_changing = matches!(
            method,
            Method::POST | Method::PUT | Method::DELETE | Method::PATCH
        );

        // Check if request is authenticated via Bearer token (API client)
        let has_bearer_auth = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.starts_with("Bearer "))
            .unwrap_or(false);

        // Extract cookie CSRF token if present
        let cookie_token = req.cookie(CSRF_COOKIE_NAME).map(|c| c.value().to_string());

        if is_state_changing && !has_bearer_auth {
            // For cookie/session-based state changes, require matching CSRF header
            let header_token = req
                .headers()
                .get(CSRF_HEADER_NAME)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string());

            match (cookie_token.as_deref(), header_token.as_deref()) {
                (Some(c), Some(h)) if !c.is_empty() && c == h => {
                    // Valid match
                }
                (Some(_), Some(_)) => {
                    return Box::pin(async move {
                        Err(ActixError::from(CsrfError::TokenMismatch))
                    });
                }
                _ => {
                    return Box::pin(async move {
                        Err(ActixError::from(CsrfError::MissingOrInvalidToken))
                    });
                }
            }
        }

        let needs_cookie = cookie_token.is_none() && !is_state_changing;
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            if needs_cookie {
                let mut bytes = [0u8; 32];
                rand::rng().fill(&mut bytes);
                let new_token = hex::encode(bytes);
                let cookie_val = format!(
                    "{}={}; Path=/; SameSite=Lax; HttpOnly=false",
                    CSRF_COOKIE_NAME, new_token
                );
                if let Ok(hdr) = header::HeaderValue::from_str(&cookie_val) {
                    res.headers_mut().append(header::SET_COOKIE, hdr);
                }
            }
            Ok(res.map_into_boxed_body())
        })
    }
}
