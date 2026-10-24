//! Hook and Extension Meta Definitions (`rwl-meta::hooks_meta`).
//!
//! Provides metadata representations for Frappe's `hooks.py` equivalents:
//! - `MethodHooks`: Document event interception (`before_insert`, `after_save`, `on_submit`, etc.).
//! - `ScheduledJob`: Periodic background jobs with interval/cron specifications.
//! - `ReportDefinition`: Standard query and script report metadata.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle document hook event types matching Frappe's `doc_events`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocEventType {
    BeforeValidate,
    Validate,
    BeforeInsert,
    AfterInsert,
    BeforeSave,
    AfterSave,
    BeforeSubmit,
    OnSubmit,
    BeforeCancel,
    OnCancel,
    OnTrash,
    AfterDelete,
}

impl DocEventType {
    /// String representation of the hook event.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BeforeValidate => "before_validate",
            Self::Validate => "validate",
            Self::BeforeInsert => "before_insert",
            Self::AfterInsert => "after_insert",
            Self::BeforeSave => "before_save",
            Self::AfterSave => "after_save",
            Self::BeforeSubmit => "before_submit",
            Self::OnSubmit => "on_submit",
            Self::BeforeCancel => "before_cancel",
            Self::OnCancel => "on_cancel",
            Self::OnTrash => "on_trash",
            Self::AfterDelete => "after_delete",
        }
    }
}

/// Registry mapping DocTypes and lifecycle events to registered handler functions/scripts.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MethodHooksRegistry {
    /// Mapping: DocType -> (DocEventType -> list of handler strings / function paths).
    hooks: HashMap<CompactString, HashMap<DocEventType, Vec<CompactString>>>,
}

impl MethodHooksRegistry {
    /// Creates a new empty `MethodHooksRegistry`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler function path for a specific DocType and lifecycle event.
    pub fn register_hook(&mut self, doctype: &str, event: DocEventType, handler_path: &str) {
        let dt_entry = self.hooks.entry(doctype.into()).or_default();
        let handlers = dt_entry.entry(event).or_default();
        if !handlers.iter().any(|h| h.as_str() == handler_path) {
            handlers.push(handler_path.into());
        }
    }

    /// Retrieves all registered handlers for a given DocType and event.
    #[must_use]
    pub fn get_handlers(&self, doctype: &str, event: DocEventType) -> &[CompactString] {
        self.hooks
            .get(doctype)
            .and_then(|m| m.get(&event))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Recurring scheduled job specification (equivalent to `scheduler_events` in Frappe).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduledJob {
    /// Unique identifier for the job.
    pub job_id: CompactString,
    /// Execution interval or cron expression (e.g. "hourly", "daily", "0 0 * * *").
    pub schedule: CompactString,
    /// Handler function or task path to execute.
    pub method: CompactString,
    /// Max execution timeout in seconds.
    #[serde(default = "default_job_timeout")]
    pub timeout_seconds: u64,
    /// Whether this job is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_job_timeout() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

/// Report type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Pure SQL / SurrealQL query report.
    QueryReport,
    /// Script-driven computational report with custom columns and data generation.
    ScriptReport,
    /// Consolidated summary report.
    SummaryReport,
}

/// Report metadata definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportDefinition {
    /// Name of the report (e.g. "General Ledger", "Stock Balance").
    pub name: CompactString,
    /// Associated reference DocType.
    pub ref_doctype: CompactString,
    /// Application module.
    pub module: CompactString,
    /// Report execution type.
    pub report_type: ReportType,
    /// Optional underlying SQL query or script identifier.
    pub query: Option<String>,
    /// Permitted roles authorized to run this report.
    pub roles: Vec<CompactString>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_hooks_registration_and_retrieval() {
        let mut registry = MethodHooksRegistry::new();
        registry.register_hook(
            "Sales Invoice",
            DocEventType::OnSubmit,
            "erp_accounting::post_gl",
        );
        registry.register_hook(
            "Sales Invoice",
            DocEventType::OnSubmit,
            "erp_trade::update_order_billed",
        );

        let handlers = registry.get_handlers("Sales Invoice", DocEventType::OnSubmit);
        assert_eq!(handlers.len(), 2);
        assert_eq!(handlers[0].as_str(), "erp_accounting::post_gl");
        assert_eq!(handlers[1].as_str(), "erp_trade::update_order_billed");

        let empty = registry.get_handlers("Sales Invoice", DocEventType::OnCancel);
        assert!(empty.is_empty());
    }
}
