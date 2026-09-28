//! Embedded Edge Analytics Engine (`erp-cms::analytics`).
//!
//! Provides an embedded, privacy-first analytics telemetry engine logging
//! page views, unique visitors, referrers, bounce rates, and device breakdown.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Analytical telemetry event record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageViewEvent {
    pub page_slug: CompactString,
    pub visitor_hash: CompactString,
    pub referrer: Option<CompactString>,
    pub user_agent_device: CompactString, // "Desktop", "Mobile", "Tablet"
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u32,
}

/// Aggregated site traffic metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsSummary {
    pub total_page_views: usize,
    pub unique_visitors: usize,
    pub bounce_rate_pct: f64,
    pub top_referrers: Vec<(CompactString, usize)>,
    pub device_breakdown: HashMap<CompactString, usize>,
}

/// In-Memory Columnar Analytics Storage Engine.
#[derive(Default)]
pub struct EdgeAnalyticsEngine {
    events: RwLock<Vec<PageViewEvent>>,
}

impl EdgeAnalyticsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new page view event.
    pub fn record_page_view(&self, event: PageViewEvent) {
        if let Ok(mut lock) = self.events.write() {
            lock.push(event);
        }
    }

    /// Computes real-time analytics aggregation summary.
    #[must_use]
    pub fn compute_summary(&self) -> AnalyticsSummary {
        let events = match self.events.read() {
            Ok(e) => e.clone(),
            Err(_) => return AnalyticsSummary {
                total_page_views: 0,
                unique_visitors: 0,
                bounce_rate_pct: 0.0,
                top_referrers: Vec::new(),
                device_breakdown: HashMap::new(),
            },
        };

        let total_page_views = events.len();
        let mut unique_visitors_set = std::collections::HashSet::new();
        let mut referrers_count: HashMap<CompactString, usize> = HashMap::new();
        let mut device_count: HashMap<CompactString, usize> = HashMap::new();
        let mut bounces = 0usize;

        for event in &events {
            unique_visitors_set.insert(event.visitor_hash.clone());

            if let Some(ref r) = event.referrer {
                *referrers_count.entry(r.clone()).or_insert(0) += 1;
            }

            *device_count.entry(event.user_agent_device.clone()).or_insert(0) += 1;

            if event.duration_seconds < 10 {
                bounces += 1;
            }
        }

        let mut top_referrers: Vec<(CompactString, usize)> = referrers_count.into_iter().collect();
        top_referrers.sort_by(|a, b| b.1.cmp(&a.1));

        let bounce_rate_pct = if total_page_views > 0 {
            (bounces as f64 / total_page_views as f64) * 100.0
        } else {
            0.0
        };

        AnalyticsSummary {
            total_page_views,
            unique_visitors: unique_visitors_set.len(),
            bounce_rate_pct,
            top_referrers,
            device_breakdown: device_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_analytics_engine() {
        let engine = EdgeAnalyticsEngine::new();

        engine.record_page_view(PageViewEvent {
            page_slug: "home".into(),
            visitor_hash: "v1".into(),
            referrer: Some("https://google.com".into()),
            user_agent_device: "Desktop".into(),
            timestamp: Utc::now(),
            duration_seconds: 45,
        });

        engine.record_page_view(PageViewEvent {
            page_slug: "products".into(),
            visitor_hash: "v1".into(),
            referrer: None,
            user_agent_device: "Desktop".into(),
            timestamp: Utc::now(),
            duration_seconds: 30,
        });

        engine.record_page_view(PageViewEvent {
            page_slug: "pricing".into(),
            visitor_hash: "v2".into(),
            referrer: Some("https://google.com".into()),
            user_agent_device: "Mobile".into(),
            timestamp: Utc::now(),
            duration_seconds: 5, // Bounce
        });

        let summary = engine.compute_summary();
        assert_eq!(summary.total_page_views, 3);
        assert_eq!(summary.unique_visitors, 2);
        assert_eq!(summary.device_breakdown.get("Desktop").copied(), Some(2));
        assert_eq!(summary.device_breakdown.get("Mobile").copied(), Some(1));
        assert_eq!(summary.top_referrers[0].0.as_str(), "https://google.com");
        assert_eq!(summary.top_referrers[0].1, 2);
    }
}
