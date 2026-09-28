//! Token Bucket Rate Limiting Subsystem (`frappe-net::rate_limit`).
//!
//! Provides brute-force IP rate-limiting and API throttling backed by atomic in-memory registers.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

/// Individual Token Bucket state for an IP or API key.
#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_update: Instant,
}

/// Token Bucket Rate Limiter.
pub struct TokenBucketRateLimiter {
    capacity: f64,
    refill_rate_per_sec: f64,
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
