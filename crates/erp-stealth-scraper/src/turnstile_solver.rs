//! WASM Proof-of-Work & Turnstile Solver Hook Integration (`erp_stealth_scraper::turnstile_solver`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Cloudflare Turnstile or DataDome challenge payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeContext {
    pub sitekey: CompactString,
    pub page_url: CompactString,
    pub action: Option<CompactString>,
    pub cdata: Option<CompactString>,
    pub timestamp_utc: i64,
}

/// Result of solved bot challenge token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeSolution {
    pub token: CompactString,
    pub user_agent: CompactString,
    pub solved_in_ms: u64,
}

/// Solves bot challenges via local WASM PoW computation or external solver webhook.
pub struct ChallengeSolverHook {
    pub solver_endpoint: Option<CompactString>,
}

impl ChallengeSolverHook {
    /// Creates a new solver hook.
    #[must_use]
    pub fn new(solver_endpoint: Option<String>) -> Self {
        Self {
            solver_endpoint: solver_endpoint.map(Into::into),
        }
    }

    /// Solves proof-of-work challenge mathematically using SHA-256.
    #[must_use]
    pub fn solve_pow_local(&self, difficulty_prefix: &str, salt: &str) -> Option<u64> {
        for nonce in 0..1_000_000u64 {
            let combined = format!("{salt}_{nonce}");
            let mut hasher = Sha256::new();
            hasher.update(combined.as_bytes());
            let result = hex::encode(hasher.finalize());
            if result.starts_with(difficulty_prefix) {
                return Some(nonce);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_pow_local() {
        let solver = ChallengeSolverHook::new(None);
        let nonce = solver.solve_pow_local("00", "test_salt_123");
        assert!(nonce.is_some());
    }
}
