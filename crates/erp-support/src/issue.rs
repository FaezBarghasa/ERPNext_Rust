//! Customer Support Issue & Ticket Lifecycle Engine (`erp-support::issue`).
//!
//! Provides canonical ERPNext Support Issue workflows:
//! - Issue state machine: Open -> Replied -> OnHold -> Resolved -> Closed (with Reopen support)
//! - Priority classification (Low, Medium, High, Urgent)
//! - Service Level Agreement (SLA) target tracking (First Response & Resolution deadlines)
//! - Resolution audit and customer sign-off.

use chrono::NaiveDate;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Support issue operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IssueError {
    #[error("Issue '{0}' is already closed")]
    AlreadyClosed(CompactString),
    #[error("Issue resolution details must be provided")]
    MissingResolution,
}

/// Lifecycle status of a Support Issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IssueStatus {
    #[default]
    Open,
    Replied,
    OnHold,
    Resolved,
    Closed,
}

/// Priority tier of an Issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum IssuePriority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

/// Service Level Agreement (SLA) status of an Issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IssueSlaStatus {
    #[default]
    WithinSla,
    SlaBreached,
}

/// Canonical Support Issue document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    pub name: CompactString,
    pub subject: CompactString,
    pub customer: CompactString,
    pub raised_by: CompactString,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub sla_status: IssueSlaStatus,
    pub opening_date: NaiveDate,
    pub response_by: Option<NaiveDate>,
    pub resolution_by: Option<NaiveDate>,
    pub first_responded_on: Option<NaiveDate>,
    pub resolution_date: Option<NaiveDate>,
    pub resolution_details: Option<CompactString>,
}

impl Issue {
    pub fn new(
        name: impl Into<CompactString>,
        subject: impl Into<CompactString>,
        customer: impl Into<CompactString>,
        raised_by: impl Into<CompactString>,
        opening_date: NaiveDate,
        priority: IssuePriority,
    ) -> Self {
        Self {
            name: name.into(),
            subject: subject.into(),
            customer: customer.into(),
            raised_by: raised_by.into(),
            status: IssueStatus::Open,
            priority,
            sla_status: IssueSlaStatus::WithinSla,
            opening_date,
            response_by: None,
            resolution_by: None,
            first_responded_on: None,
            resolution_date: None,
            resolution_details: None,
        }
    }

    /// Sets SLA response and resolution targets.
    pub fn set_sla_targets(&mut self, response_by: NaiveDate, resolution_by: NaiveDate) {
        self.response_by = Some(response_by);
        self.resolution_by = Some(resolution_by);
    }

    /// Records agent reply, transitioning state to Replied and checking response SLA.
    pub fn record_reply(&mut self, reply_date: NaiveDate) {
        if self.first_responded_on.is_none() {
            self.first_responded_on = Some(reply_date);
            if self.response_by.is_some_and(|target| reply_date > target) {
                self.sla_status = IssueSlaStatus::SlaBreached;
            }
        }
        if self.status == IssueStatus::Open {
            self.status = IssueStatus::Replied;
        }
    }

    /// Resolves the issue with resolution notes and verifies resolution deadline compliance.
    pub fn resolve(
        &mut self,
        resolution: impl Into<CompactString>,
        resolved_date: NaiveDate,
    ) -> Result<(), IssueError> {
        let details = resolution.into();
        if details.trim().is_empty() {
            return Err(IssueError::MissingResolution);
        }

        self.resolution_details = Some(details);
        self.resolution_date = Some(resolved_date);
        self.status = IssueStatus::Resolved;

        if self
            .resolution_by
            .is_some_and(|target| resolved_date > target)
        {
            self.sla_status = IssueSlaStatus::SlaBreached;
        }

        Ok(())
    }

    /// Closes the issue after customer confirmation.
    pub fn close(&mut self) -> Result<(), IssueError> {
        if self.status == IssueStatus::Closed {
            return Err(IssueError::AlreadyClosed(self.name.clone()));
        }
        self.status = IssueStatus::Closed;
        Ok(())
    }

    /// Reopens an issue if the customer reports recurring trouble.
    pub fn reopen(&mut self) {
        self.status = IssueStatus::Open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_lifecycle_within_sla() {
        let open_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let resp_target = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap();
        let res_target = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();

        let mut issue = Issue::new(
            "ISS-2026-0001",
            "Inverter displaying Error Code E-42",
            "Solar Dynamics Corp",
            "tech@solardynamics.com",
            open_date,
            IssuePriority::High,
        );

        issue.set_sla_targets(resp_target, res_target);
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

        // Reply on same day -> Within SLA
        issue.record_reply(open_date);
        assert_eq!(issue.status, IssueStatus::Replied);
        assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

        // Resolve on Oct 19 -> Before Oct 20 deadline
        let resolve_date = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap();
        issue
            .resolve(
                "Updated firmware to v3.12 which patched RS485 communication timeout",
                resolve_date,
            )
            .unwrap();

        assert_eq!(issue.status, IssueStatus::Resolved);
        assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

        // Close
        issue.close().unwrap();
        assert_eq!(issue.status, IssueStatus::Closed);
    }

    #[test]
    fn test_issue_sla_breached() {
        let open_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let resp_target = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap();
        let res_target = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();

        let mut issue = Issue::new(
            "ISS-2026-0002",
            "Hydraulic Leak on Machine #2",
            "Heavy Industries",
            "plant@heavy.local",
            open_date,
            IssuePriority::Urgent,
        );

        issue.set_sla_targets(resp_target, res_target);

        // Resolved late on Oct 25 -> Breached!
        let late_resolve = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        issue
            .resolve("Replaced main cylinder seal gasket", late_resolve)
            .unwrap();

        assert_eq!(issue.status, IssueStatus::Resolved);
        assert_eq!(issue.sla_status, IssueSlaStatus::SlaBreached);
    }
}
