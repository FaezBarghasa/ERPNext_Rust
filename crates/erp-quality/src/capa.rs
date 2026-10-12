//! Corrective and Preventive Actions (CAPA) Module (`erp-quality::capa`).
//!
//! Provides closed-loop quality management system (QMS) remediation:
//! - Root cause investigation (5 Whys, Ishikawa)
//! - Corrective Action vs Preventive Action tracking
//! - Verifiable action items with assigned owners and due dates
//! - Post-implementation effectiveness review

use chrono::NaiveDate;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// CAPA Action Category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapaActionType {
    Corrective,
    Preventive,
    Both,
}

/// Lifecycle status of a CAPA workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapaStatus {
    Initiated,
    InvestigationUnderway,
    ActionPlanApproved,
    InImplementation,
    PendingEffectivenessReview,
    Closed,
}

/// An individual action task inside a CAPA remediation plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapaActionItem {
    pub item_id: u32,
    pub description: CompactString,
    pub assigned_to: CompactString,
    pub target_date: NaiveDate,
    pub completed_date: Option<NaiveDate>,
    pub is_completed: bool,
}

impl CapaActionItem {
    #[must_use]
    pub fn new(id: u32, description: &str, assigned_to: &str, target_date: NaiveDate) -> Self {
        Self {
            item_id: id,
            description: description.into(),
            assigned_to: assigned_to.into(),
            target_date,
            completed_date: None,
            is_completed: false,
        }
    }

    pub fn complete(&mut self, date: NaiveDate) {
        self.completed_date = Some(date);
        self.is_completed = true;
    }
}

/// Complete CAPA record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapaRecord {
    /// Document identifier (e.g. "CAPA-2026-00001").
    pub name: CompactString,
    /// Associated NCR reference if triggered by a non-conformance.
    pub ncr_ref: Option<CompactString>,
    /// Action category.
    pub action_type: CapaActionType,
    /// Detailed problem statement.
    pub problem_statement: CompactString,
    /// Detailed root cause findings.
    pub root_cause_analysis: CompactString,
    /// Remediation action items.
    pub action_items: Vec<CapaActionItem>,
    /// Date when effectiveness review is scheduled.
    pub review_date: Option<NaiveDate>,
    /// Effectiveness evaluation notes.
    pub effectiveness_review: Option<CompactString>,
    /// Current workflow status.
    pub status: CapaStatus,
}

impl CapaRecord {
    /// Initializes a new CAPA record.
    #[must_use]
    pub fn new(
        name: &str,
        ncr_ref: Option<&str>,
        action_type: CapaActionType,
        problem: &str,
        root_cause: &str,
    ) -> Self {
        Self {
            name: name.into(),
            ncr_ref: ncr_ref.map(Into::into),
            action_type,
            problem_statement: problem.into(),
            root_cause_analysis: root_cause.into(),
            action_items: Vec::new(),
            review_date: None,
            effectiveness_review: None,
            status: CapaStatus::Initiated,
        }
    }

    /// Adds a planned action item.
    pub fn add_action_item(&mut self, item: CapaActionItem) {
        self.action_items.push(item);
    }

    /// Approves the action plan and transitions to implementation.
    pub fn approve_plan(&mut self) {
        self.status = CapaStatus::InImplementation;
    }

    /// Checks if all action items are completed, moving to effectiveness review.
    pub fn check_implementation_complete(&mut self) -> bool {
        if !self.action_items.is_empty() && self.action_items.iter().all(|i| i.is_completed) {
            self.status = CapaStatus::PendingEffectivenessReview;
            true
        } else {
            false
        }
    }

    /// Conducts the final review and closes the CAPA.
    pub fn close_with_effectiveness_review(&mut self, review_notes: &str, review_date: NaiveDate) {
        self.effectiveness_review = Some(review_notes.into());
        self.review_date = Some(review_date);
        self.status = CapaStatus::Closed;
    }
}
