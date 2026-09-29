//! Security Headers Middleware (`frappe-net::middleware::security_headers`).
//!
//! Injects industry-standard OWASP security response headers across all HTTP endpoints:
//! - `Content-Security-Policy` (CSP)
//! - `Strict-Transport-Security` (HSTS)
//! - `X-Content-Type-Options: nosniff`
//! - `X-Frame-Options: SAMEORIGIN`
//! - `X-XSS-Protection: 1; mode=block`
//! - `Referrer-Policy: strict-origin-when-cross-origin`
//! - `Permissions-Policy: camera=(), microphone=(), geolocation=()`

use actix_web::Error as ActixError;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use futures_util::future::{LocalBoxFuture, Ready, ok};

/// Security Headers Middleware Config.
#[derive(Clone, Debug)]
pub struct SecurityHeaders {
    csp: String,
    hsts: String,
    x_frame_options: String,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            csp: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self' wss: ws: https:; frame-ancestors 'self';".to_string(),
            hsts: "max-age=63072000; includeSubDomains; preload".to_string(),
            x_frame_options: "SAMEORIGIN".to_string(),
        }
    }
}

impl SecurityHeaders {
    /// Creates a new default security headers middleware.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows custom Content-Security-Policy definition.
    #[must_use]
    pub fn with_csp(mut self, csp: &str) -> Self {
        self.csp = csp.to_string();
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = SecurityHeadersService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityHeadersService {
            service,
            config: self.clone(),
        })
    }
}

pub struct SecurityHeadersService<S> {
    service: S,
    config: SecurityHeaders,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
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
        let config = self.config.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            // Insert Security Headers safely
            if let Ok(val) = HeaderValue::from_str(&config.csp) {
                headers.insert(HeaderName::from_static("content-security-policy"), val);
            }
            if let Ok(val) = HeaderValue::from_str(&config.hsts) {
                headers.insert(HeaderName::from_static("strict-transport-security"), val);
            }
            if let Ok(val) = HeaderValue::from_str(&config.x_frame_options) {
                headers.insert(HeaderName::from_static("x-frame-options"), val);
            }

            headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
            );

            Ok(res.map_into_boxed_body())
        })
    }
}
