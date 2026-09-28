//! Asynchronous Priority Task Queue & Worker Pool (`frappe-net::queue`).
//!
//! Replaces Redis/Celery with Tokio-native prioritized asynchronous worker pools,
//! bounded zero-allocation channels, exponential backoff retries, and task state transitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Priority task dispatcher managing dedicated Tokio worker pools.
#[derive(Clone)]
pub struct PriorityTaskDispatcher {
    critical_tx: mpsc::Sender<BackgroundJob>,
    high_tx: mpsc::Sender<BackgroundJob>,
    default_tx: mpsc::Sender<BackgroundJob>,
    low_tx: mpsc::Sender<BackgroundJob>,
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
        };

        (dispatcher, crit_rx, high_rx, def_rx, low_rx)
    }

    /// Enqueues a job into its respective priority channel asynchronously.
    pub async fn enqueue(&self, job: BackgroundJob) -> Result<(), QueueError> {
        let priority = job.priority;
        let res = match priority {
            PriorityLevel::Critical => self.critical_tx.send(job).await,
            PriorityLevel::High => self.high_tx.send(job).await,
            PriorityLevel::Default => self.default_tx.send(job).await,
            PriorityLevel::Low => self.low_tx.send(job).await,
        };

        res.map_err(|_| QueueError::ChannelClosed(priority))
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

    #[tokio::test]
    async fn test_priority_dispatcher() {
        let (dispatcher, mut crit_rx, _high_rx, _def_rx, mut low_rx) =
            PriorityTaskDispatcher::new(16);

        let job_crit = BackgroundJob::new(
            "job-crit".into(),
            "t1".into(),
            PriorityLevel::Critical,
            "gl_post".into(),
            "{}".into(),
        );
        let job_low = BackgroundJob::new(
            "job-low".into(),
            "t1".into(),
            PriorityLevel::Low,
            "cleanup".into(),
            "{}".into(),
        );

        assert!(dispatcher.enqueue(job_crit).await.is_ok());
        assert!(dispatcher.enqueue(job_low).await.is_ok());

        let received_crit = crit_rx.recv().await.unwrap();
        assert_eq!(received_crit.id, "job-crit");
        assert_eq!(received_crit.priority, PriorityLevel::Critical);

        let received_low = low_rx.recv().await.unwrap();
        assert_eq!(received_low.id, "job-low");
        assert_eq!(received_low.priority, PriorityLevel::Low);
    }
}
