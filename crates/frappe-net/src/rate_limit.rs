//! Token Bucket Rate Limiting Subsystem (`frappe-net::rate_limit`).
//!
//! Provides brute-force IP rate-limiting and API throttling backed by atomic in-memory registers
//! and Actix Web middleware integration.

use actix_web::body::{BoxBody, MessageBody};
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
    B: MessageBody + 'static,
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
            return Box::pin(ready(Ok(ServiceResponse::new(
                http_req,
                res.map_into_boxed_body(),
            ))));
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

/// Brute-force Login Protection & Lockout Guard.
/// Tuple tracking: (failure_count, first_failure_instant, optional_lockout_until)
pub type LockoutAttemptRecord = (u32, Instant, Option<Instant>);

/// Sliding-window brute-force lockout guard.
pub struct LoginGuard {
    max_failures: u32,
    failure_window: std::time::Duration,
    lockout_duration: std::time::Duration,
    attempts: RwLock<HashMap<String, LockoutAttemptRecord>>,
}

impl Default for LoginGuard {
    fn default() -> Self {
        Self::new(5, 900, 1800) // 5 failures in 15 mins -> 30 min lockout
    }
}

impl LoginGuard {
    /// Creates a new login guard instance.
    #[must_use]
    pub fn new(max_failures: u32, window_secs: u64, lockout_secs: u64) -> Self {
        Self {
            max_failures,
            failure_window: std::time::Duration::from_secs(window_secs),
            lockout_duration: std::time::Duration::from_secs(lockout_secs),
            attempts: RwLock::new(HashMap::new()),
        }
    }

    /// Checks if a client IP or username is currently locked out.
    /// Returns `Ok(())` if allowed, or `Err(remaining_seconds)` if locked out.
    pub fn check_allowed(&self, ip: &str, username: &str) -> Result<(), u64> {
        let now = Instant::now();
        let attempts = match self.attempts.read() {
            Ok(a) => a,
            Err(_) => return Ok(()),
        };

        for key in &[format!("ip:{ip}"), format!("user:{username}")] {
            if let Some((_, _, Some(locked_until))) = attempts.get(key)
                && now < *locked_until
            {
                let remaining = locked_until.duration_since(now).as_secs().max(1);
                return Err(remaining);
            }
        }

        Ok(())
    }

    /// Records a failed login attempt for both IP and username.
    /// Returns (max_attempt_count, is_now_locked).
    pub fn record_failure(&self, ip: &str, username: &str) -> (u32, bool) {
        let now = Instant::now();
        let mut attempts = match self.attempts.write() {
            Ok(a) => a,
            Err(_) => return (1, false),
        };

        let mut max_count = 0;
        let mut locked = false;

        for key in &[format!("ip:{ip}"), format!("user:{username}")] {
            let entry = attempts.entry(key.clone()).or_insert((0, now, None));

            // Reset count if window has expired
            if now.duration_since(entry.1) > self.failure_window && entry.2.is_none() {
                entry.0 = 0;
                entry.1 = now;
            }

            entry.0 += 1;
            entry.1 = now;
            max_count = max_count.max(entry.0);

            if entry.0 >= self.max_failures {
                entry.2 = Some(now + self.lockout_duration);
                locked = true;
            }
        }

        (max_count, locked)
    }

    /// Resets failed attempt counters upon successful login.
    pub fn record_success(&self, ip: &str, username: &str) {
        if let Ok(mut attempts) = self.attempts.write() {
            attempts.remove(&format!("ip:{ip}"));
            attempts.remove(&format!("user:{username}"));
        }
    }

    /// Explicitly unlocks a target key (e.g. administrative override).
    pub fn unlock_target(&self, target: &str) {
        if let Ok(mut attempts) = self.attempts.write() {
            attempts.remove(&format!("ip:{target}"));
            attempts.remove(&format!("user:{target}"));
            attempts.remove(target);
        }
    }

    /// Lists all currently active lockouts with remaining seconds.
    #[must_use]
    pub fn list_locked_targets(&self) -> Vec<(String, u64)> {
        let now = Instant::now();
        let mut locked = Vec::new();
        if let Ok(attempts) = self.attempts.read() {
            for (key, (_, _, lockout)) in attempts.iter() {
                if let Some(locked_until) = *lockout
                    && now < locked_until
                {
                    let remaining = locked_until.duration_since(now).as_secs().max(1);
                    locked.push((key.clone(), remaining));
                }
            }
        }
        locked
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

    #[test]
    fn test_login_guard_lockout_and_reset() {
        let guard = LoginGuard::new(3, 10, 60);
        let ip = "10.0.0.5";
        let user = "admin@example.com";

        // Initially allowed
        assert!(guard.check_allowed(ip, user).is_ok());

        // 1st failure
        let (count, locked) = guard.record_failure(ip, user);
        assert_eq!(count, 1);
        assert!(!locked);
        assert!(guard.check_allowed(ip, user).is_ok());

        // 2nd failure
        let (count, locked) = guard.record_failure(ip, user);
        assert_eq!(count, 2);
        assert!(!locked);

        // 3rd failure triggers lockout
        let (count, locked) = guard.record_failure(ip, user);
        assert_eq!(count, 3);
        assert!(locked);

        // Now blocked
        assert!(guard.check_allowed(ip, user).is_err());

        // Successful login or admin unlock resets
        guard.record_success(ip, user);
        assert!(guard.check_allowed(ip, user).is_ok());
    }
}
