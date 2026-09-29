use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Proxy health status and backoff metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub url: CompactString,
    pub failure_count: usize,
    pub success_count: usize,
    pub last_latency_ms: u64,
    pub is_quarantined: bool,
}

/// Thread-safe proxy rotation ring with exponential cooldown.
pub struct ProxyRotationRing {
    nodes: RwLock<Vec<ProxyNode>>,
    cursor: AtomicUsize,
    quarantine_duration: Duration,
}

impl ProxyRotationRing {
    /// Returns the configured quarantine duration.
    #[must_use]
    pub fn quarantine_duration(&self) -> Duration {
        self.quarantine_duration
    }
    /// Creates a new proxy pool with specified quarantine timeout.
    #[must_use]
    pub fn new(proxies: Vec<String>, quarantine_secs: u64) -> Self {
        let nodes = proxies
            .into_iter()
            .map(|url| ProxyNode {
                url: url.into(),
                failure_count: 0,
                success_count: 0,
                last_latency_ms: 0,
                is_quarantined: false,
            })
            .collect();

        Self {
            nodes: RwLock::new(nodes),
            cursor: AtomicUsize::new(0),
            quarantine_duration: Duration::from_secs(quarantine_secs),
        }
    }

    /// Retrieves the next available healthy proxy in round-robin sequence.
    pub fn next_healthy_proxy(&self) -> Option<CompactString> {
        let nodes = self.nodes.read().ok()?;
        if nodes.is_empty() {
            return None;
        }

        let total = nodes.len();
        for _ in 0..total {
            let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % total;
            if let Some(node) = nodes.get(idx)
                && !node.is_quarantined
            {
                return Some(node.url.clone());
            }
        }

        // Fallback: return any proxy if all are quarantined
        nodes.first().map(|n| n.url.clone())
    }

    /// Records a successful scraping operation on a proxy.
    pub fn record_success(&self, proxy_url: &str, latency_ms: u64) {
        if let Ok(mut nodes) = self.nodes.write()
            && let Some(node) = nodes.iter_mut().find(|n| n.url.as_str() == proxy_url)
        {
            node.success_count += 1;
            node.failure_count = 0;
            node.last_latency_ms = latency_ms;
            node.is_quarantined = false;
        }
    }

    /// Records a failure or rate limit and triggers exponential quarantine backoff.
    pub fn record_failure(&self, proxy_url: &str) {
        if let Ok(mut nodes) = self.nodes.write()
            && let Some(node) = nodes.iter_mut().find(|n| n.url.as_str() == proxy_url)
        {
            node.failure_count += 1;
            if node.failure_count >= 3 {
                node.is_quarantined = true;
            }
        }
    }

    /// Total number of configured proxies.
    pub fn len(&self) -> usize {
        self.nodes.read().map(|n| n.len()).unwrap_or(0)
    }

    /// Returns whether the proxy ring is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_rotation_and_quarantine() {
        let ring = ProxyRotationRing::new(
            vec!["http://p1.node:8080".into(), "http://p2.node:8080".into()],
            60,
        );

        let p1 = ring.next_healthy_proxy().unwrap();
        assert_eq!(p1, "http://p1.node:8080");

        let p2 = ring.next_healthy_proxy().unwrap();
        assert_eq!(p2, "http://p2.node:8080");

        // Fail p1 3 times -> quarantined
        ring.record_failure("http://p1.node:8080");
        ring.record_failure("http://p1.node:8080");
        ring.record_failure("http://p1.node:8080");

        // Next proxy should strictly be p2
        let next = ring.next_healthy_proxy().unwrap();
        assert_eq!(next, "http://p2.node:8080");
    }
}
