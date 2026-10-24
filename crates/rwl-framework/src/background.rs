//! Background Job Queue & Task Scheduler (`frappe-framework::background`).
//!
//! Provides priority job dispatching (Critical, Default, Long), retry semantics,
//! failure diagnostics, and periodic recurring scheduler triggers (Every5Min, Hourly, Daily, Weekly, Monthly).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Priority level governing worker dequeue order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    Long = 0,
    Default = 1,
    Critical = 2,
}

/// Execution status of an asynchronous background job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

/// Queued background work item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackgroundJob {
    pub id: String,
    pub task_name: String,
    pub payload: serde_json::Value,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error: Option<String>,
    pub enqueued_at: String,
    pub completed_at: Option<String>,
}

/// Thread-safe in-memory background job queue.
#[derive(Default)]
pub struct BackgroundJobQueue {
    queue: Mutex<VecDeque<BackgroundJob>>,
    jobs: Mutex<Vec<BackgroundJob>>,
}

impl BackgroundJobQueue {
    /// Creates a new background job queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues a new background job.
    pub fn enqueue(
        &self,
        task_name: &str,
        payload: serde_json::Value,
        priority: JobPriority,
    ) -> String {
        let job_id = format!("JOB-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let job = BackgroundJob {
            id: job_id.clone(),
            task_name: task_name.to_string(),
            payload,
            priority,
            status: JobStatus::Queued,
            retry_count: 0,
            max_retries: 3,
            error: None,
            enqueued_at: Utc::now().to_rfc3339(),
            completed_at: None,
        };

        let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        // Insert maintaining priority descending
        let pos = q
            .iter()
            .position(|j| j.priority < priority)
            .unwrap_or(q.len());
        q.insert(pos, job.clone());

        let mut store = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        store.push(job);

        job_id
    }

    /// Dequeues the highest priority job ready for execution.
    pub fn dequeue(&self) -> Option<BackgroundJob> {
        let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        let mut job = q.pop_front()?;
        job.status = JobStatus::Running;
        Some(job)
    }

    /// Marks a job as successfully completed.
    pub fn complete(&self, job_id: &str) {
        let mut store = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(j) = store.iter_mut().find(|j| j.id == job_id) {
            j.status = JobStatus::Completed;
            j.completed_at = Some(Utc::now().to_rfc3339());
        }
    }

    /// Marks a job as failed, auto-retrying if under max_retries limit.
    pub fn fail(&self, job_id: &str, error: &str) {
        let mut store = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(j) = store.iter_mut().find(|j| j.id == job_id) {
            j.retry_count += 1;
            j.error = Some(error.to_string());

            if j.retry_count <= j.max_retries {
                j.status = JobStatus::Queued;
                let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
                q.push_back(j.clone());
            } else {
                j.status = JobStatus::Failed;
                j.completed_at = Some(Utc::now().to_rfc3339());
            }
        }
    }

    /// Fetches job details by ID.
    #[must_use]
    pub fn get(&self, job_id: &str) -> Option<BackgroundJob> {
        let store = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        store.iter().find(|j| j.id == job_id).cloned()
    }
}

/// Standard recurring scheduler intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerInterval {
    Every5Minutes,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl SchedulerInterval {
    /// Returns interval duration in seconds.
    #[must_use]
    pub fn as_duration(&self) -> Duration {
        match self {
            Self::Every5Minutes => Duration::minutes(5),
            Self::Hourly => Duration::hours(1),
            Self::Daily => Duration::days(1),
            Self::Weekly => Duration::weeks(1),
            Self::Monthly => Duration::days(30),
        }
    }
}

/// Scheduled cron job registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub name: String,
    pub interval: SchedulerInterval,
    pub last_run: Option<DateTime<Utc>>,
}

/// Recurring scheduler engine tracking task execution readiness.
#[derive(Default)]
pub struct SchedulerEngine {
    tasks: Vec<ScheduledTask>,
}

impl SchedulerEngine {
    /// Creates a new scheduler engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a recurring task.
    pub fn register(&mut self, name: &str, interval: SchedulerInterval) {
        self.tasks.push(ScheduledTask {
            name: name.to_string(),
            interval,
            last_run: None,
        });
    }

    /// Evaluates which tasks are due for execution at the provided timestamp.
    pub fn tick(&mut self, now: DateTime<Utc>) -> Vec<String> {
        let mut ready = Vec::new();
        for task in &mut self.tasks {
            let is_due = match task.last_run {
                None => true,
                Some(last) => now - last >= task.interval.as_duration(),
            };

            if is_due {
                ready.push(task.name.clone());
                task.last_run = Some(now);
            }
        }
        ready
    }
}

/// Coordinator orchestrating periodic scheduler evaluation and background job dispatch.
pub struct JobMeshCoordinator {
    pub queue: Arc<BackgroundJobQueue>,
    pub scheduler: Mutex<SchedulerEngine>,
}

impl JobMeshCoordinator {
    /// Creates a new JobMesh coordinator.
    #[must_use]
    pub fn new(queue: Arc<BackgroundJobQueue>, scheduler: SchedulerEngine) -> Self {
        Self {
            queue,
            scheduler: Mutex::new(scheduler),
        }
    }

    /// Evaluates scheduled tasks and enqueues due tasks into the worker queue.
    pub fn tick_scheduler_and_enqueue(&self, now: DateTime<Utc>) -> Vec<String> {
        let mut sched = self.scheduler.lock().unwrap_or_else(|e| e.into_inner());
        let ready_tasks = sched.tick(now);
        for task in &ready_tasks {
            self.queue.enqueue(
                task,
                serde_json::json!({
                    "scheduled": true,
                    "tick_time": now.to_rfc3339(),
                }),
                JobPriority::Default,
            );
        }
        ready_tasks
    }

    /// Drains ready jobs from the queue and executes the provided handler.
    pub fn drain_and_process<F>(&self, max_jobs: usize, mut handler: F) -> usize
    where
        F: FnMut(&BackgroundJob) -> Result<(), String>,
    {
        let mut processed = 0;
        while processed < max_jobs {
            let Some(job) = self.queue.dequeue() else {
                break;
            };

            match handler(&job) {
                Ok(()) => {
                    self.queue.complete(&job.id);
                }
                Err(err) => {
                    self.queue.fail(&job.id, &err);
                }
            }
            processed += 1;
        }
        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_queue_priority_and_retry() {
        let queue = BackgroundJobQueue::new();

        queue.enqueue(
            "generate_reports",
            serde_json::json!({}),
            JobPriority::Default,
        );
        queue.enqueue(
            "urgent_gl_posting",
            serde_json::json!({}),
            JobPriority::Critical,
        );
        queue.enqueue("clean_temp_files", serde_json::json!({}), JobPriority::Long);

        // First dequeued must be Critical
        let job1 = queue.dequeue().expect("Critical job must dequeue first");
        assert_eq!(job1.task_name, "urgent_gl_posting");
        queue.complete(&job1.id);
        assert_eq!(queue.get(&job1.id).unwrap().status, JobStatus::Completed);

        // Next must be Default
        let job2 = queue.dequeue().expect("Default job must dequeue second");
        assert_eq!(job2.task_name, "generate_reports");

        // Fail job2 -> should requeue for retry
        queue.fail(&job2.id, "Database timeout");
        let retried_job = queue.get(&job2.id).unwrap();
        assert_eq!(retried_job.status, JobStatus::Queued);
        assert_eq!(retried_job.retry_count, 1);
    }

    #[test]
    fn test_scheduler_interval_evaluation() {
        let mut scheduler = SchedulerEngine::new();
        scheduler.register("daily_revaluation", SchedulerInterval::Daily);

        let t0 = Utc::now();
        let ready0 = scheduler.tick(t0);
        assert_eq!(ready0, vec!["daily_revaluation"]);

        // Immediately after -> not ready
        let ready1 = scheduler.tick(t0 + Duration::hours(12));
        assert!(ready1.is_empty());

        // 25 hours later -> ready again!
        let ready2 = scheduler.tick(t0 + Duration::hours(25));
        assert_eq!(ready2, vec!["daily_revaluation"]);
    }

    #[test]
    fn test_job_mesh_coordinator_tick_and_drain() {
        let queue = Arc::new(BackgroundJobQueue::new());
        let mut scheduler = SchedulerEngine::new();
        scheduler.register("hourly_sync", SchedulerInterval::Hourly);

        let coordinator = JobMeshCoordinator::new(queue.clone(), scheduler);
        let now = Utc::now();

        let enqueued = coordinator.tick_scheduler_and_enqueue(now);
        assert_eq!(enqueued, vec!["hourly_sync"]);

        let mut executed = Vec::new();
        let count = coordinator.drain_and_process(10, |job| {
            executed.push(job.task_name.clone());
            Ok(())
        });

        assert_eq!(count, 1);
        assert_eq!(executed, vec!["hourly_sync"]);
    }
}
