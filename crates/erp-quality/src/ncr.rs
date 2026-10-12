//! Non-Conformance Report (NCR) Module (`erp-quality::ncr`).
//!
//! Provides statutory ISO 9001 non-conformance tracking:
//! - Discrepancy identification and quarantine
//! - Severity classification (Critical, Major, Minor)
//! - Containment actions and material dispositioning (Rework, Scrap, Return, Use As-Is)

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Severity classification of the non-conformance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NcrSeverity {
    Critical,
    Major,
    Minor,
}

/// Material disposition decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NcrDisposition {
    UnderReview,
    Rework,
    Scrap,
    ReturnToSupplier,
    UseAsIs,
}

/// Lifecycle status of an NCR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NcrStatus {
    Open,
    UnderInvestigation,
    DispositionDecided,
    CapaRequired,
    Closed,
}

/// Complete Non-Conformance Report document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonConformanceReport {
    /// Document identifier (e.g. "NCR-2026-00001").
    pub name: CompactString,
    /// Target item code with defect.
    pub item_code: CompactString,
    /// Batch or serial reference if applicable.
    pub batch_no: Option<CompactString>,
    /// Quantity affected.
    pub quantity_affected: Decimal,
    /// Date of occurrence or identification.
    pub issue_date: NaiveDate,
    /// Severity level.
    pub severity: NcrSeverity,
    /// Descriptive summary of the defect.
    pub defect_description: CompactString,
    /// Immediate containment action to prevent propagation.
    pub containment_action: CompactString,
    /// Root cause analysis findings.
    pub root_cause: Option<CompactString>,
    /// Disposition action decided.
    pub disposition: NcrDisposition,
    /// Inspection reference if raised from Quality Inspection.
    pub inspection_ref: Option<CompactString>,
    /// Associated CAPA document if initiated.
    pub capa_ref: Option<CompactString>,
    /// Current status.
    pub status: NcrStatus,
}

impl NonConformanceReport {
    /// Creates a new open NCR.
    #[must_use]
    pub fn new(
        name: &str,
        item_code: &str,
        qty: Decimal,
        issue_date: NaiveDate,
        severity: NcrSeverity,
        description: &str,
        containment: &str,
    ) -> Self {
        Self {
            name: name.into(),
            item_code: item_code.into(),
            batch_no: None,
            quantity_affected: qty,
            issue_date,
            severity,
            defect_description: description.into(),
            containment_action: containment.into(),
            root_cause: None,
            disposition: NcrDisposition::UnderReview,
            inspection_ref: None,
            capa_ref: None,
            status: NcrStatus::Open,
        }
    }

    /// Sets the disposition and transitions to `DispositionDecided`.
    pub fn decide_disposition(&mut self, disposition: NcrDisposition, root_cause: &str) {
        self.disposition = disposition;
        self.root_cause = Some(root_cause.into());
        self.status = NcrStatus::DispositionDecided;
    }

    /// Links a CAPA and marks as requiring corrective action.
    pub fn link_capa(&mut self, capa_name: &str) {
        self.capa_ref = Some(capa_name.into());
        self.status = NcrStatus::CapaRequired;
    }

    /// Closes the NCR once disposition and actions are fulfilled.
    pub fn close(&mut self) {
        self.status = NcrStatus::Closed;
    }
}
