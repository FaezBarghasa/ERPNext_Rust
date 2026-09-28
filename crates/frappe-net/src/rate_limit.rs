//! Token Bucket Rate Limiting Subsystem (`frappe-net::rate_limit`).
//!
//! Provides brute-force IP rate-limiting and API throttling backed by atomic in-memory registers
//! and Actix Web middleware integration.

use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::{Error as ActixError, HttpResponse};
use futures_util::future::{LocalBoxFuture, Ready, ok, ready};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Individual Token Bucket state for an IP or API key.
#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_update: Instant,
}

/// Token Bucket Rate Limiter.
#[derive(Debug)]
pub struct TokenBucketRateLimiter {
    pub capacity: f64,
    pub refill_rate_per_sec: f64,
    buckets: RwLock<HashMap<String, Bucket>>,
}

impl TokenBucketRateLimiter {
    /// Creates a new TokenBucketRateLimiter with maximum burst capacity and refill rate.
    #[must_use]
    pub fn new(capacity: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            capacity,
            refill_rate_per_sec,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Attempts to consume tokens for a given identifier (e.g. IP address).
    /// Returns true if permitted, false if rate limited.
    pub fn allow(&self, key: &str, cost: f64) -> bool {
        let now = Instant::now();
        let mut buckets = match self.buckets.write() {
            Ok(b) => b,
            Err(_) => return true, // Fail open on lock poisoning
        };

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: self.capacity,
            last_update: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_update).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate_per_sec).min(self.capacity);
        bucket.last_update = now;

        if bucket.tokens >= cost {
            bucket.tokens -= cost;
            true
        } else {
            false
        }
    }
}

/// Actix Web Middleware for Rate Limiting.
#[derive(Clone, Debug)]
pub struct RateLimitMiddleware {
    limiter: Arc<TokenBucketRateLimiter>,
}

impl RateLimitMiddleware {
    /// Creates a new rate limiting middleware instance.
    #[must_use]
    pub fn new(capacity: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            limiter: Arc::new(TokenBucketRateLimiter::new(capacity, refill_rate_per_sec)),
        }
    }

    /// Creates middleware sharing an existing limiter.
    #[must_use]
    pub fn from_limiter(limiter: Arc<TokenBucketRateLimiter>) -> Self {
        Self { limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = RateLimitService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitService {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitService<S> {
    service: S,
    limiter: Arc<TokenBucketRateLimiter>,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
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
        let client_ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                req.connection_info()
                    .peer_addr()
                    .map(|p| p.split(':').next().unwrap_or(p).to_string())
            })
            .unwrap_or_else(|| "127.0.0.1".to_string());

        if !self.limiter.allow(&client_ip, 1.0) {
            let res = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                .insert_header(("Retry-After", "60"))
                .insert_header(("X-RateLimit-Limit", self.limiter.capacity.to_string()))
                .json(serde_json::json!({
                    "error": "Too Many Requests: Rate limit exceeded"
                }));
            let (http_req, _) = req.into_parts();
            return Box::pin(ready(Ok(
                ServiceResponse::new(http_req, res.map_into_boxed_body())
            )));
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_rate_limiter() {
        let limiter = TokenBucketRateLimiter::new(3.0, 1.0);
        let client_ip = "192.168.1.100";

        // Consume 3 tokens immediately (capacity = 3)
        assert!(limiter.allow(client_ip, 1.0));
        assert!(limiter.allow(client_ip, 1.0));
        assert!(limiter.allow(client_ip, 1.0));

        // 4th request must be rejected
        assert!(!limiter.allow(client_ip, 1.0));
    }
}
