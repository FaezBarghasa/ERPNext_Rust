//! Cross-Origin Resource Sharing (CORS) Middleware (`frappe-net::middleware::cors`).
//!
//! Provides enterprise CORS preflight (`OPTIONS`) handling and response headers
//! for single-page applications, mobile apps, and multi-tenant storefront domains.

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::{Error as ActixError, HttpResponse};
use futures_util::future::{LocalBoxFuture, Ready, ok, ready};

/// Configuration for Enterprise CORS Middleware.
#[derive(Clone, Debug)]
pub struct CorsMiddleware {
    allowed_methods: String,
    allowed_headers: String,
    exposed_headers: String,
    max_age_secs: u32,
    allow_credentials: bool,
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self {
            allowed_methods: "GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD".to_string(),
            allowed_headers:
                "Authorization, Content-Type, Accept, X-Tenant-Id, X-Requested-With, X-Request-Id"
                    .to_string(),
            exposed_headers:
                "X-Request-Id, Content-Length, Content-Type, Content-Disposition, Alt-Svc"
                    .to_string(),
            max_age_secs: 86400,
            allow_credentials: true,
        }
    }
}

impl CorsMiddleware {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_methods(mut self, methods: &str) -> Self {
        self.allowed_methods = methods.to_string();
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for CorsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = CorsService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CorsService {
            service,
            config: self.clone(),
        })
    }
}

pub struct CorsService<S> {
    service: S,
    config: CorsMiddleware,
}

impl<S, B> Service<ServiceRequest> for CorsService<S>
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
        let origin = req
            .headers()
            .get("Origin")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "*".to_string());

        let is_options = req.method() == Method::OPTIONS;
        let config = self.config.clone();

        if is_options {
            // Immediate Preflight Response
            let mut res = HttpResponse::build(StatusCode::NO_CONTENT);
            if let Ok(val) = HeaderValue::from_str(&origin) {
                res.insert_header((HeaderName::from_static("access-control-allow-origin"), val));
            }
            if let Ok(val) = HeaderValue::from_str(&config.allowed_methods) {
                res.insert_header((HeaderName::from_static("access-control-allow-methods"), val));
            }
            if let Ok(val) = HeaderValue::from_str(&config.allowed_headers) {
                res.insert_header((HeaderName::from_static("access-control-allow-headers"), val));
            }
            if config.allow_credentials && origin != "*" {
                res.insert_header((
                    HeaderName::from_static("access-control-allow-credentials"),
                    HeaderValue::from_static("true"),
                ));
            }
            res.insert_header((
                HeaderName::from_static("access-control-max-age"),
                HeaderValue::from_str(&config.max_age_secs.to_string())
                    .unwrap_or(HeaderValue::from_static("86400")),
            ));

            let (http_req, _) = req.into_parts();
            return Box::pin(ready(Ok(ServiceResponse::new(
                http_req,
                res.finish().map_into_boxed_body(),
            ))));
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            if let Ok(val) = HeaderValue::from_str(&origin) {
                headers.insert(HeaderName::from_static("access-control-allow-origin"), val);
            }
            if let Ok(val) = HeaderValue::from_str(&config.exposed_headers) {
                headers.insert(
                    HeaderName::from_static("access-control-expose-headers"),
                    val,
                );
            }
            if config.allow_credentials && origin != "*" {
                headers.insert(
                    HeaderName::from_static("access-control-allow-credentials"),
                    HeaderValue::from_static("true"),
                );
            }

            Ok(res.map_into_boxed_body())
        })
    }
}
