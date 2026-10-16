//! Upstream ERPNext Project & Task Management Engine (`erp-ppm::project`).
//!
//! Provides canonical ERPNext project management:
//! - Project lifecycle: Open -> InProgress -> Completed -> OnHold / Cancelled
//! - Work breakdown structure (WBS) Tasks with dependency ordering
//! - Task progress calculation and project-level rollup
//! - Timesheet actual cost & logged hours integration.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// PPM operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PpmError {
    #[error("Task '{0}' has circular dependency")]
    CircularDependency(CompactString),
    #[error("Project '{0}' is already completed")]
    ProjectAlreadyCompleted(CompactString),
    #[error("Invalid progress percentage: {0} (must be between 0.0 and 100.0)")]
    InvalidProgress(Decimal),
}

/// Project lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectStatus {
    #[default]
    Open,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

/// Task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    #[default]
    Open,
    Working,
    PendingReview,
    Completed,
    Cancelled,
}

/// Individual Project Task line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub task_id: CompactString,
    pub project: CompactString,
    pub subject: CompactString,
    pub status: TaskStatus,
    pub expected_hours: Decimal,
    pub actual_hours: Decimal,
    pub progress: Decimal, // 0.0 to 100.0
    pub depends_on: Vec<CompactString>,
}

impl Task {
    pub fn new(
        task_id: impl Into<CompactString>,
        project: impl Into<CompactString>,
        subject: impl Into<CompactString>,
        expected_hours: Decimal,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            project: project.into(),
            subject: subject.into(),
            status: TaskStatus::Open,
            expected_hours,
            actual_hours: Decimal::ZERO,
            progress: Decimal::ZERO,
            depends_on: Vec::new(),
        }
    }

    /// Logs worked hours onto this task.
    pub fn log_hours(&mut self, hours: Decimal) {
        self.actual_hours += hours;
        if self.status == TaskStatus::Open {
            self.status = TaskStatus::Working;
        }
    }

    /// Updates task progress percentage (0.0 - 100.0).
    pub fn update_progress(&mut self, progress: Decimal) -> Result<(), PpmError> {
        if progress < Decimal::ZERO || progress > dec!(100.0) {
            return Err(PpmError::InvalidProgress(progress));
        }
        self.progress = progress;
        if progress == dec!(100.0) {
            self.status = TaskStatus::Completed;
        } else if self.status == TaskStatus::Open && progress > Decimal::ZERO {
            self.status = TaskStatus::Working;
        }
        Ok(())
    }
}

/// Project document in ERPNext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub name: CompactString,
    pub project_name: CompactString,
    pub status: ProjectStatus,
    pub customer: Option<CompactString>,
    pub expected_start_date: Option<NaiveDate>,
    pub expected_end_date: Option<NaiveDate>,
    pub estimated_cost: Decimal,
    pub total_cost: Decimal,
    pub total_billed_amount: Decimal,
    pub percent_complete: Decimal,
}

impl Project {
    pub fn new(
        name: impl Into<CompactString>,
        project_name: impl Into<CompactString>,
        estimated_cost: Decimal,
    ) -> Self {
        Self {
            name: name.into(),
            project_name: project_name.into(),
            status: ProjectStatus::Open,
            customer: None,
            expected_start_date: None,
            expected_end_date: None,
            estimated_cost,
            total_cost: Decimal::ZERO,
            total_billed_amount: Decimal::ZERO,
            percent_complete: Decimal::ZERO,
        }
    }

    /// Recalculates project percentage completion weighted by task expected hours.
    pub fn recalculate_progress_from_tasks(&mut self, tasks: &[Task]) {
        let matching_tasks: Vec<&Task> = tasks.iter().filter(|t| t.project == self.name).collect();

        if matching_tasks.is_empty() {
            self.percent_complete = Decimal::ZERO;
            return;
        }

        let total_expected_hours: Decimal = matching_tasks.iter().map(|t| t.expected_hours).sum();

        if total_expected_hours.is_zero() {
            // Unweighted average
            let total_progress: Decimal = matching_tasks.iter().map(|t| t.progress).sum();
            self.percent_complete =
                (total_progress / Decimal::from(matching_tasks.len())).round_dp(2);
        } else {
            // Weighted average by expected hours
            let weighted_sum: Decimal = matching_tasks
                .iter()
                .map(|t| t.expected_hours * t.progress)
                .sum();
            self.percent_complete = (weighted_sum / total_expected_hours).round_dp(2);
        }

        if self.percent_complete == dec!(100.0) {
            self.status = ProjectStatus::Completed;
        } else if self.percent_complete > Decimal::ZERO && self.status == ProjectStatus::Open {
            self.status = ProjectStatus::InProgress;
        }
    }

    /// Records accumulated labor timesheet or expense cost into the project.
    pub fn record_cost(&mut self, cost: Decimal) {
        self.total_cost += cost;
    }

    /// Records client billing invoice against this project.
    pub fn record_billing(&mut self, billed_amount: Decimal) {
        self.total_billed_amount += billed_amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_task_weighted_progress_rollup() {
        let mut project = Project::new("PROJ-001", "Next-Gen Drone Firmware", dec!(50000.0));
        assert_eq!(project.status, ProjectStatus::Open);

        // Task 1: 40 hours (80% of project)
        let mut t1 = Task::new("TASK-01", "PROJ-001", "RTIC v2 Drivers", dec!(40.0));
        // Task 2: 10 hours (20% of project)
        let t2 = Task::new("TASK-02", "PROJ-001", "HIL Test Suite", dec!(10.0));

        // Progress: T1 is 50% done, T2 is 0% done
        t1.update_progress(dec!(50.0)).unwrap();
        t1.log_hours(dec!(20.0));

        let tasks = vec![t1.clone(), t2.clone()];
        project.recalculate_progress_from_tasks(&tasks);

        // Weighted progress: (40 * 50 + 10 * 0) / 50 = 2000 / 50 = 40.0%
        assert_eq!(project.percent_complete, dec!(40.0));
        assert_eq!(project.status, ProjectStatus::InProgress);

        // Now complete both tasks
        let mut t1_done = t1;
        t1_done.update_progress(dec!(100.0)).unwrap();
        let mut t2_done = t2;
        t2_done.update_progress(dec!(100.0)).unwrap();

        let tasks_done = vec![t1_done, t2_done];
        project.recalculate_progress_from_tasks(&tasks_done);

        assert_eq!(project.percent_complete, dec!(100.0));
        assert_eq!(project.status, ProjectStatus::Completed);
    }
}
