use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

/// HR & Attendance errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HrError {
    /// Ingestion buffer full.
    #[error("Biometric ingestion queue channel full")]
    IngestionQueueFull,
    /// Insufficient remaining leave balance.
    #[error(
        "Insufficient leave balance for {leave_type}: requested {requested}, available {available}"
    )]
    InsufficientLeaveBalance {
        leave_type: String,
        requested: rust_decimal::Decimal,
        available: rust_decimal::Decimal,
    },
    /// Overlapping leave application.
    #[error("Overlapping leave application exists between {from_date} and {to_date}")]
    OverlappingLeave {
        from_date: NaiveDate,
        to_date: NaiveDate,
    },
    /// Employee calculation failed.
    #[error("Salary calculation failed for employee '{0}': {1}")]
    SalaryCalculationError(String, String),
}

/// Raw biometric timestamp check-in / check-out punch log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BiometricPunch {
    /// Employee identifier.
    pub employee_id: String,
    /// Device identifier.
    pub device_id: String,
    /// Timestamp of punch.
    pub timestamp_date: NaiveDate,
    /// Exact punch time.
    pub punch_time: NaiveTime,
}

/// High-throughput Biometric Ingestion Gateway (Milestone 3.5).
pub struct BiometricIngestionGateway {
    sender: mpsc::Sender<BiometricPunch>,
}

impl BiometricIngestionGateway {
    /// Creates a bounded 10,000 capacity ingestion gateway.
    #[must_use]
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<BiometricPunch>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { sender }, receiver)
    }

    /// Ingests an incoming punch event without blocking server request threads.
    pub fn ingest_punch(&self, punch: BiometricPunch) -> Result<(), HrError> {
        self.sender
            .try_send(punch)
            .map_err(|_| HrError::IngestionQueueFull)
    }
}

/// Shift specification with grace period margins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShiftType {
    /// Shift name (e.g. "General Morning").
    pub name: String,
    /// Shift start time.
    pub start_time: NaiveTime,
    /// Shift end time.
    pub end_time: NaiveTime,
    /// Late entry grace period in minutes.
    pub late_entry_grace_mins: u32,
    /// Early exit grace period in minutes.
    pub early_exit_grace_mins: u32,
}

/// Reconciled daily attendance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttendanceStatus {
    Present,
    LateEntry,
    EarlyExit,
    HalfDay,
    Absent,
}

/// Shift Reconciliation Engine (Milestone 3.5).
pub struct AttendanceReconciler;

impl AttendanceReconciler {
    /// Reconciles daily check-in and check-out against shift boundaries.
    #[must_use]
    pub fn reconcile_shift(
        shift: &ShiftType,
        check_in: Option<NaiveTime>,
        check_out: Option<NaiveTime>,
    ) -> AttendanceStatus {
        let (in_time, out_time) = match (check_in, check_out) {
            (Some(i), Some(o)) => (i, o),
            (Some(_), None) | (None, Some(_)) => return AttendanceStatus::HalfDay,
            (None, None) => return AttendanceStatus::Absent,
        };

        let shift_duration_mins = (shift.end_time - shift.start_time).num_minutes();
        let worked_mins = (out_time - in_time).num_minutes();

        if worked_mins < shift_duration_mins / 2 {
            return AttendanceStatus::HalfDay;
        }

        let is_late =
            (in_time - shift.start_time).num_minutes() > shift.late_entry_grace_mins as i64;
        let is_early_exit =
            (shift.end_time - out_time).num_minutes() > shift.early_exit_grace_mins as i64;

        if is_late && is_early_exit {
            AttendanceStatus::HalfDay
        } else if is_late {
            AttendanceStatus::LateEntry
        } else if is_early_exit {
            AttendanceStatus::EarlyExit
        } else {
            AttendanceStatus::Present
        }
    }
}
