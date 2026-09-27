use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Streaming and CMS security errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StreamingError {
    /// Invalid HTTP Byte Range header format.
    #[error("Invalid Range header: {0}")]
    InvalidRangeHeader(String),
    /// Range requested is unsatisfiable.
    #[error("Range not satisfiable: start {start}, end {end}, total {total}")]
    RangeNotSatisfiable { start: u64, end: u64, total: u64 },
    /// Invalid HMAC access signature.
    #[error("Invalid or forged HMAC streaming token")]
    InvalidSignature,
    /// Token expired.
    #[error("Streaming token has expired (expiry {expiry_timestamp}, current {current_timestamp})")]
    TokenExpired {
        expiry_timestamp: u64,
        current_timestamp: u64,
    },
    /// SVoD Subscription inactive.
    #[error("Active SVoD subscription required for media playback")]
    SubscriptionRequired,
    /// Maximum concurrent playback streams reached.
    #[error("Stream limit exceeded: maximum {max_streams} concurrent streams allowed")]
    StreamLimitExceeded { max_streams: usize },
}

/// HTTP 206 Byte Range Parser (Milestone 5.4).
pub fn parse_byte_range(range_header: &str, total_size: u64) -> Result<(u64, u64), StreamingError> {
    let trimmed = range_header.trim();
    if !trimmed.starts_with("bytes=") {
        return Err(StreamingError::InvalidRangeHeader(range_header.to_string()));
    }

    let range_str = &trimmed[6..];
    let mut parts = range_str.split('-');

    let start_str = parts
        .next()
        .ok_or_else(|| StreamingError::InvalidRangeHeader(range_header.to_string()))?;
    let end_str = parts.next().unwrap_or("");

    let (start, end) = if start_str.is_empty() {
        if end_str.is_empty() {
            return Err(StreamingError::InvalidRangeHeader(range_header.to_string()));
        }
        let suffix_len: u64 = end_str
            .parse()
            .map_err(|_| StreamingError::InvalidRangeHeader(range_header.to_string()))?;
        let s = total_size.saturating_sub(suffix_len);
        let e = total_size.saturating_sub(1);
        (s, e)
    } else {
        let s: u64 = start_str
            .parse()
            .map_err(|_| StreamingError::InvalidRangeHeader(range_header.to_string()))?;
        let e: u64 = if end_str.is_empty() {
            total_size.saturating_sub(1)
        } else {
            end_str
                .parse()
                .map_err(|_| StreamingError::InvalidRangeHeader(range_header.to_string()))?
        };
        (s, e)
    };

    if start > end || start >= total_size {
        return Err(StreamingError::RangeNotSatisfiable {
            start,
            end,
            total: total_size,
        });
    }

    let clamped_end = end.min(total_size.saturating_sub(1));
    Ok((start, clamped_end))
}

/// Signed HMAC-SHA256 Token Generator & Validator (Milestone 5.4).
pub struct HmacStreamingSigner {
    secret_key: Vec<u8>,
}

impl HmacStreamingSigner {
    /// Creates a new signer with the specified secret key.
    #[must_use]
    pub fn new(secret_key: &[u8]) -> Self {
        Self {
            secret_key: secret_key.to_vec(),
        }
    }

    /// Generates a signed token string: `hex(hmac(media_id || expiry))`.
    pub fn generate_token(&self, media_id: &str, expiry_timestamp: u64) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .expect("HMAC can take key of any size");
        let payload = format!("{media_id}:{expiry_timestamp}");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Validates an incoming signed token against media ID and current timestamp.
    pub fn verify_token(
        &self,
        media_id: &str,
        expiry_timestamp: u64,
        current_timestamp: u64,
        provided_token: &str,
    ) -> Result<(), StreamingError> {
        if current_timestamp > expiry_timestamp {
            return Err(StreamingError::TokenExpired {
                expiry_timestamp,
                current_timestamp,
            });
        }

        let expected_token = self.generate_token(media_id, expiry_timestamp);
        if expected_token != provided_token {
            return Err(StreamingError::InvalidSignature);
        }

        Ok(())
    }
}

/// SVoD Subscription Gating & Real-Time Playback Concurrency Limiter (Milestone 5.5).
#[derive(Default, Clone)]
pub struct SvodPlaybackManager {
    /// User ID -> List of active streaming session tokens.
    active_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SvodPlaybackManager {
    /// Creates a new playback manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a playback session if subscription is active and concurrency limit is not breached.
    pub fn start_playback(
        &self,
        user_id: &str,
        session_id: String,
        is_subscription_active: bool,
        max_concurrent_streams: usize,
    ) -> Result<(), StreamingError> {
        if !is_subscription_active {
            return Err(StreamingError::SubscriptionRequired);
        }

        let mut lock = self
            .active_sessions
            .write()
            .map_err(|_| StreamingError::SubscriptionRequired)?;

        let sessions = lock.entry(user_id.to_string()).or_default();
        if sessions.len() >= max_concurrent_streams {
            return Err(StreamingError::StreamLimitExceeded {
                max_streams: max_concurrent_streams,
            });
        }

        sessions.push(session_id);
        Ok(())
    }

    /// Ends an active playback session.
    pub fn end_playback(&self, user_id: &str, session_id: &str) {
        if let Ok(mut lock) = self.active_sessions.write() {
            if let Some(sessions) = lock.get_mut(user_id) {
                sessions.retain(|s| s != session_id);
            }
        }
    }
}

mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        data.as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
