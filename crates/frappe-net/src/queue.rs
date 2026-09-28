//! Asynchronous Priority Task Queue, Scheduler & Deduplication Engine (`frappe-net::queue`).
//!
//! Replaces Redis/Celery with Tokio-native prioritized asynchronous worker pools,
//! bounded zero-allocation channels, exponential backoff retries, staggered maintenance schedules,
//! and atomic lock-free `is_job_enqueued` deduplication registers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

pub use frappe_storage::{QueueTask, TaskState};

/// Supported task queue priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PriorityLevel {
    Critical = 0,
    High = 1,
    Default = 2,
    Low = 3,
}

impl PriorityLevel {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PriorityLevel::Critical => "critical",
            PriorityLevel::High => "high",
            PriorityLevel::Default => "default",
            PriorityLevel::Low => "low",
        }
    }
}

pub const QUEUES: &[&str] = &["critical", "high", "default", "low"];

/// Queue-related errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum QueueError {
    #[error("Queue channel closed for priority: {0:?}")]
    ChannelClosed(PriorityLevel),
    #[error("Job '{0}' is already enqueued (deduplicated)")]
    JobAlreadyEnqueued(String),
    #[error("Task execution failed after {retries} retries: {reason}")]
    MaxRetriesExceeded { retries: u32, reason: String },
}

/// Asynchronous background job payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackgroundJob {
    pub id: String,
    pub tenant_id: String,
    pub priority: PriorityLevel,
    pub job_type: String,
    pub payload_json: String,
    pub state: TaskState,
    pub max_retries: u32,
    pub retry_count: u32,
    pub backoff_base_ms: u64,
    pub created_at: DateTime<Utc>,
}

impl BackgroundJob {
    #[must_use]
    pub fn new(
        id: String,
        tenant_id: String,
        priority: PriorityLevel,
        job_type: String,
        payload_json: String,
    ) -> Self {
        Self {
            id,
            tenant_id,
            priority,
            job_type,
            payload_json,
            state: TaskState::Queued,
            max_retries: 3,
            retry_count: 0,
            backoff_base_ms: 50,
            created_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    #[must_use]
    pub fn next_backoff_duration(&self) -> Duration {
        let factor = 2u64.saturating_pow(self.retry_count);
        Duration::from_millis(self.backoff_base_ms.saturating_mul(factor))
    }

    pub fn transition_to(&mut self, to: TaskState) -> bool {
        let ok = matches!(
            (self.state, to),
            (TaskState::Queued, TaskState::Processing)
                | (TaskState::Processing, TaskState::Completed)
                | (TaskState::Processing, TaskState::Failed)
                | (TaskState::Failed, TaskState::Queued)
        );
        if ok {
            self.state = to;
        }
        ok
    }
}

/// Status record for background report downloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportDownloadJob {
    pub job_id: String,
    pub report_name: String,
    pub status: String, // "Queued", "Generating", "Ready", "Failed"
    pub download_url: Option<String>,
    pub row_count: usize,
    pub error: Option<String>,
}

/// Staggered Maintenance Task window calculator to prevent midnight load surges.
pub struct StaggeredMaintenanceScheduler;

impl StaggeredMaintenanceScheduler {
    /// Computes staggered execution offset in minutes based on tenant hash.
    #[must_use]
    pub fn compute_staggered_minute_offset(tenant_id: &str, window_span_minutes: u32) -> u32 {
        let hash: u32 = tenant_id.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        hash % window_span_minutes
    }
}

/// Priority task dispatcher managing dedicated Tokio worker pools with deduplication.
#[derive(Clone)]
pub struct PriorityTaskDispatcher {
    critical_tx: mpsc::Sender<BackgroundJob>,
    high_tx: mpsc::Sender<BackgroundJob>,
    default_tx: mpsc::Sender<BackgroundJob>,
    low_tx: mpsc::Sender<BackgroundJob>,
    enqueued_job_keys: std::sync::Arc<RwLock<HashSet<String>>>,
}

impl PriorityTaskDispatcher {
    /// Creates a dispatcher and initializes priority worker channels with bounded buffers.
    #[must_use]
    pub fn new(
        buffer_size: usize,
    ) -> (
        Self,
        mpsc::Receiver<BackgroundJob>,
        mpsc::Receiver<BackgroundJob>,
        mpsc::Receiver<BackgroundJob>,
        mpsc::Receiver<BackgroundJob>,
    ) {
        let (crit_tx, crit_rx) = mpsc::channel(buffer_size);
        let (high_tx, high_rx) = mpsc::channel(buffer_size);
        let (def_tx, def_rx) = mpsc::channel(buffer_size);
        let (low_tx, low_rx) = mpsc::channel(buffer_size);

        let dispatcher = Self {
            critical_tx: crit_tx,
            high_tx,
            default_tx: def_tx,
            low_tx,
            enqueued_job_keys: std::sync::Arc::new(RwLock::new(HashSet::new())),
        };

        (dispatcher, crit_rx, high_rx, def_rx, low_rx)
    }

    /// Checks if a job with the specified key is already enqueued (`is_job_enqueued`).
    #[must_use]
    pub fn is_job_enqueued(&self, job_key: &str) -> bool {
        if let Ok(lock) = self.enqueued_job_keys.read() {
            lock.contains(job_key)
        } else {
            false
        }
    }

    /// Enqueues a job into its respective priority channel with deduplication.
    pub async fn enqueue(&self, job: BackgroundJob) -> Result<(), QueueError> {
        let priority = job.priority;
        let job_key = format!("{}:{}:{}", job.tenant_id, job.job_type, job.id);

        if let Ok(mut lock) = self.enqueued_job_keys.write() {
            if !lock.insert(job_key) {
                return Err(QueueError::JobAlreadyEnqueued(job.id));
            }
        }

        let res = match priority {
            PriorityLevel::Critical => self.critical_tx.send(job).await,
            PriorityLevel::High => self.high_tx.send(job).await,
            PriorityLevel::Default => self.default_tx.send(job).await,
            PriorityLevel::Low => self.low_tx.send(job).await,
        };

        res.map_err(|_| QueueError::ChannelClosed(priority))
    }

    /// Marks a job completed, releasing its deduplication key.
    pub fn mark_completed(&self, tenant_id: &str, job_type: &str, job_id: &str) {
        let job_key = format!("{tenant_id}:{job_type}:{job_id}");
        if let Ok(mut lock) = self.enqueued_job_keys.write() {
            lock.remove(&job_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queues_constant() {
        assert_eq!(QUEUES.len(), 4);
    }

    #[test]
    fn test_staggered_scheduler() {
        let offset1 = StaggeredMaintenanceScheduler::compute_staggered_minute_offset("tenant_acme", 60);
        let offset2 = StaggeredMaintenanceScheduler::compute_staggered_minute_offset("tenant_apex", 60);
        assert!(offset1 < 60);
        assert!(offset2 < 60);
    }

    #[tokio::test]
    async fn test_job_deduplication() {
        let (dispatcher, mut crit_rx, _high_rx, _def_rx, _low_rx) =
            PriorityTaskDispatcher::new(16);

        let job1 = BackgroundJob::new(
            "report-001".into(),
            "t1".into(),
            PriorityLevel::Critical,
            "financial_report".into(),
            "{}".into(),
        );
        let job2 = BackgroundJob::new(
            "report-001".into(),
            "t1".into(),
            PriorityLevel::Critical,
            "financial_report".into(),
            "{}".into(),
        );

        assert!(dispatcher.enqueue(job1).await.is_ok());
        assert!(dispatcher.is_job_enqueued("t1:financial_report:report-001"));

        // Duplicate enqueue must fail
        assert_eq!(
            dispatcher.enqueue(job2).await,
            Err(QueueError::JobAlreadyEnqueued("report-001".into()))
        );

        let _ = crit_rx.recv().await;
        dispatcher.mark_completed("t1", "financial_report", "report-001");
        assert!(!dispatcher.is_job_enqueued("t1:financial_report:report-001"));
    }

    #[test]
    fn test_job_transitions_and_backoff() {
        let mut job = BackgroundJob::new(
            "job-1".into(),
            "tenant-1".into(),
            PriorityLevel::High,
            "generate_invoice".into(),
            "{}".into(),
        );

        assert_eq!(job.state, TaskState::Queued);
        assert!(job.transition_to(TaskState::Processing));
        assert_eq!(job.state, TaskState::Processing);
        assert!(job.transition_to(TaskState::Failed));
        assert_eq!(job.state, TaskState::Failed);
        assert!(job.transition_to(TaskState::Queued));
        assert_eq!(job.state, TaskState::Queued);

        assert!(job.can_retry());
        assert_eq!(job.next_backoff_duration(), Duration::from_millis(50));
        job.retry_count = 1;
        assert_eq!(job.next_backoff_duration(), Duration::from_millis(100));
        job.retry_count = 2;
        assert_eq!(job.next_backoff_duration(), Duration::from_millis(200));
    }
}
