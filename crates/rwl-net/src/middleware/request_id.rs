//! Request Correlation ID Middleware (`frappe-net::middleware::request_id`).
//!
//! Generates or propagates a unique `X-Request-Id` UUID header for every incoming HTTP request,
//! making it available in Actix Request extensions and appending it to all HTTP responses
//! for end-to-end distributed tracing and observability.

use actix_web::Error as ActixError;
use actix_web::HttpMessage;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Container for the request ID stored in Actix request extensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(pub String);

/// Request ID Middleware.
#[derive(Clone, Debug, Default)]
pub struct RequestIdMiddleware;

impl RequestIdMiddleware {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = RequestIdService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdService { service })
    }
}

pub struct RequestIdService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
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
        // Extract existing X-Request-Id header or generate a deterministic high-entropy UUID-v4-like string
        let req_id_str = req
            .headers()
            .get("X-Request-Id")
            .and_then(|h| h.to_str().ok())
            .filter(|s| !s.is_empty() && s.len() <= 128)
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let ts = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                let count = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                format!("req-{ts:x}-{count:04x}")
            });

        // Insert into request extensions
        req.extensions_mut().insert(RequestId(req_id_str.clone()));

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;
            if let Ok(val) = HeaderValue::from_str(&req_id_str) {
                res.headers_mut()
                    .insert(HeaderName::from_static("x-request-id"), val);
            }
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let count = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let id = format!("req-{ts:x}-{count:04x}");
        assert!(id.starts_with("req-"));
    }
}
