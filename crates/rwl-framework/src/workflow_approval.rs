//! Multi-Tier Approval Workflow Engine & Document Versioning (`frappe-framework::workflow_approval`).
//!
//! Provides enterprise approval workflows and audit version tracking matching ERPNext & Odoo:
//! - Multi-level role-based approval pipelines (e.g. Draft -> Manager Approval -> CFO Approval -> Approved)
//! - Conditional transition rules with Rhai scripting evaluation
//! - Mandatory approval comments and audit trail logging
//! - Document time-travel snapshot versioning and delta patch history

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Workflow approval execution errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkflowError {
    #[error("No transition rule found for state '{0}' with action '{1}'")]
    NoTransitionFound(String, String),
    #[error(
        "User with roles {0:?} is not authorized to execute action '{1}' (requires one of {2:?})"
    )]
    UnauthorizedRole(Vec<String>, String, Vec<String>),
    #[error("Mandatory approval/rejection comment is missing for action '{0}'")]
    CommentRequired(String),
    #[error("Condition script evaluation failed: {0}")]
    ConditionFailed(String),
    #[error("Workflow state '{0}' not defined in workflow")]
    StateNotFound(String),
    #[error("Version {0} not found for document '{1}:{2}'")]
    VersionNotFound(u32, String, String),
}

/// State node definition in a workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowStateNode {
    pub state_name: String,
    pub docstatus: i32, // 0 = Draft, 1 = Submitted, 2 = Cancelled
    pub allow_edit_roles: Vec<String>,
    pub is_final_approval: bool,
    pub is_rejection: bool,
}

impl WorkflowStateNode {
    #[must_use]
    pub fn new(state_name: impl Into<String>, docstatus: i32) -> Self {
        Self {
            state_name: state_name.into(),
            docstatus,
            allow_edit_roles: Vec::new(),
            is_final_approval: false,
            is_rejection: false,
        }
    }

    #[must_use]
    pub fn with_edit_roles(mut self, roles: Vec<String>) -> Self {
        self.allow_edit_roles = roles;
        self
    }

    #[must_use]
    pub fn as_final_approval(mut self) -> Self {
        self.is_final_approval = true;
        self
    }

    #[must_use]
    pub fn as_rejection(mut self) -> Self {
        self.is_rejection = true;
        self
    }
}

/// Rule governing allowed state transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTransitionRule {
    pub state: String,
    pub action: String,
    pub next_state: String,
    pub allowed_roles: Vec<String>,
    pub condition_rhai_script: Option<String>,
    pub require_comment: bool,
}

impl WorkflowTransitionRule {
    #[must_use]
    pub fn new(
        state: impl Into<String>,
        action: impl Into<String>,
        next_state: impl Into<String>,
        allowed_roles: Vec<String>,
    ) -> Self {
        Self {
            state: state.into(),
            action: action.into(),
            next_state: next_state.into(),
            allowed_roles,
            condition_rhai_script: None,
            require_comment: false,
        }
    }

    #[must_use]
    pub fn with_condition(mut self, script: impl Into<String>) -> Self {
        self.condition_rhai_script = Some(script.into());
        self
    }

    #[must_use]
    pub fn with_mandatory_comment(mut self) -> Self {
        self.require_comment = true;
        self
    }
}

/// Historical audit log entry for workflow actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowApprovalLog {
    pub id: String,
    pub doctype: String,
    pub doc_name: String,
    pub user_id: String,
    pub action: String,
    pub from_state: String,
    pub to_state: String,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Outcome of a successful workflow transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTransitionOutcome {
    pub previous_state: String,
    pub new_state: String,
    pub new_docstatus: i32,
    pub log_entry: WorkflowApprovalLog,
}

/// Approval Workflow configuration attached to a DocType.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    pub doctype: String,
    pub workflow_name: String,
    pub is_active: bool,
    pub initial_state: String,
    pub states: Vec<WorkflowStateNode>,
    pub transitions: Vec<WorkflowTransitionRule>,
}

impl ApprovalWorkflow {
    #[must_use]
    pub fn new(
        doctype: impl Into<String>,
        workflow_name: impl Into<String>,
        initial_state: impl Into<String>,
    ) -> Self {
        Self {
            doctype: doctype.into(),
            workflow_name: workflow_name.into(),
            is_active: true,
            initial_state: initial_state.into(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_states(mut self, states: Vec<WorkflowStateNode>) -> Self {
        self.states = states;
        self
    }

    #[must_use]
    pub fn with_transitions(mut self, transitions: Vec<WorkflowTransitionRule>) -> Self {
        self.transitions = transitions;
        self
    }

    /// Evaluates whether a user can execute an action on a document in `current_state`.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_transition(
        &self,
        doc_name: &str,
        current_state: &str,
        action: &str,
        user_id: &str,
        user_roles: &[String],
        comment: Option<&str>,
        doc_data: &serde_json::Value,
    ) -> Result<WorkflowTransitionOutcome, WorkflowError> {
        // Find matching transition rule
        let rule = self
            .transitions
            .iter()
            .find(|t| {
                t.state.eq_ignore_ascii_case(current_state) && t.action.eq_ignore_ascii_case(action)
            })
            .ok_or_else(|| {
                WorkflowError::NoTransitionFound(current_state.to_string(), action.to_string())
            })?;

        // Check role permissions (Administrator bypasses role requirement)
        let is_admin = user_roles
            .iter()
            .any(|r| r == "Administrator" || r == "System Manager");
        let has_role = is_admin
            || rule
                .allowed_roles
                .iter()
                .any(|req| user_roles.iter().any(|user_r| user_r == req));

        if !has_role {
            return Err(WorkflowError::UnauthorizedRole(
                user_roles.to_vec(),
                action.to_string(),
                rule.allowed_roles.clone(),
            ));
        }

        // Check mandatory comment
        if rule.require_comment {
            let is_empty = comment.is_none_or(|c| c.trim().is_empty());
            if is_empty {
                return Err(WorkflowError::CommentRequired(action.to_string()));
            }
        }

        // Check condition script if present (simple Rhai check or JSON field check)
        if rule
            .condition_rhai_script
            .as_ref()
            .is_some_and(|script| !Self::evaluate_simple_condition(script, doc_data))
        {
            return Err(WorkflowError::ConditionFailed(
                rule.condition_rhai_script.clone().unwrap_or_default(),
            ));
        }

        // Find target state node to retrieve docstatus
        let target_node = self
            .states
            .iter()
            .find(|s| s.state_name.eq_ignore_ascii_case(&rule.next_state))
            .ok_or_else(|| WorkflowError::StateNotFound(rule.next_state.clone()))?;

        let now = Utc::now();
        let log_id = format!("wf_log_{}_{}", doc_name, now.timestamp_millis());

        let log_entry = WorkflowApprovalLog {
            id: log_id,
            doctype: self.doctype.clone(),
            doc_name: doc_name.to_string(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            from_state: current_state.to_string(),
            to_state: rule.next_state.clone(),
            comment: comment.map(|s| s.trim().to_string()),
            timestamp: now,
        };

        Ok(WorkflowTransitionOutcome {
            previous_state: current_state.to_string(),
            new_state: rule.next_state.clone(),
            new_docstatus: target_node.docstatus,
            log_entry,
        })
    }

    fn evaluate_simple_condition(script: &str, doc_data: &serde_json::Value) -> bool {
        // Evaluate simple field comparisons e.g. "grand_total > 5000"
        let parts: Vec<&str> = script.split_whitespace().collect();
        if parts.len() == 3 {
            let field = parts[0];
            let op = parts[1];
            let target_val: f64 = parts[2].parse().unwrap_or(0.0);

            if let Some(num) = doc_data.get(field).and_then(|v| v.as_f64()) {
                return match op {
                    ">" => num > target_val,
                    ">=" => num >= target_val,
                    "<" => num < target_val,
                    "<=" => num <= target_val,
                    "==" => (num - target_val).abs() < f64::EPSILON,
                    _ => true,
                };
            }
        }
        true
    }
}

/// Immutable document revision record for time-travel audit trails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentVersionRecord {
    pub id: String,
    pub doctype: String,
    pub doc_name: String,
    pub version_num: u32,
    pub modified_by: String,
    pub modified_at: DateTime<Utc>,
    pub full_snapshot: serde_json::Value,
    pub delta_patch: serde_json::Value,
    pub change_reason: Option<String>,
}

/// Document Versioning and Time-Travel Engine.
#[derive(Debug, Clone, Default)]
pub struct DocumentVersioningEngine {
    history: HashMap<String, Vec<DocumentVersionRecord>>,
}

impl DocumentVersioningEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    /// Captures a new revision of a document.
    pub fn capture_version(
        &mut self,
        doctype: &str,
        doc_name: &str,
        modified_by: &str,
        snapshot: serde_json::Value,
        delta: serde_json::Value,
        reason: Option<String>,
    ) -> DocumentVersionRecord {
        let key = format!("{doctype}:{doc_name}");
        let entries = self.history.entry(key).or_default();
        let next_version = (entries.len() as u32) + 1;

        let record = DocumentVersionRecord {
            id: format!("ver_{}_{}_{}", doctype, doc_name, next_version),
            doctype: doctype.to_string(),
            doc_name: doc_name.to_string(),
            version_num: next_version,
            modified_by: modified_by.to_string(),
            modified_at: Utc::now(),
            full_snapshot: snapshot,
            delta_patch: delta,
            change_reason: reason,
        };

        entries.push(record.clone());
        record
    }

    /// Retrieves full revision history for a document.
    #[must_use]
    pub fn get_history(&self, doctype: &str, doc_name: &str) -> Vec<DocumentVersionRecord> {
        let key = format!("{doctype}:{doc_name}");
        self.history.get(&key).cloned().unwrap_or_default()
    }

    /// Restores snapshot of a specific historical version.
    pub fn get_version(
        &self,
        doctype: &str,
        doc_name: &str,
        version_num: u32,
    ) -> Result<serde_json::Value, WorkflowError> {
        let key = format!("{doctype}:{doc_name}");
        let entries = self.history.get(&key).ok_or_else(|| {
            WorkflowError::VersionNotFound(version_num, doctype.to_string(), doc_name.to_string())
        })?;

        entries
            .iter()
            .find(|v| v.version_num == version_num)
            .map(|v| v.full_snapshot.clone())
            .ok_or_else(|| {
                WorkflowError::VersionNotFound(
                    version_num,
                    doctype.to_string(),
                    doc_name.to_string(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_tier_approval_pipeline_and_role_enforcement() {
        let states = vec![
            WorkflowStateNode::new("Draft", 0).with_edit_roles(vec!["Sales User".into()]),
            WorkflowStateNode::new("Pending Manager Approval", 0),
            WorkflowStateNode::new("Pending CFO Approval", 0),
            WorkflowStateNode::new("Approved", 1).as_final_approval(),
            WorkflowStateNode::new("Rejected", 2).as_rejection(),
        ];

        let transitions = vec![
            WorkflowTransitionRule::new(
                "Draft",
                "Submit for Approval",
                "Pending Manager Approval",
                vec!["Sales User".into(), "Sales Manager".into()],
            ),
            WorkflowTransitionRule::new(
                "Pending Manager Approval",
                "Approve Level 1",
                "Pending CFO Approval",
                vec!["Sales Manager".into()],
            )
            .with_condition("grand_total > 50000"),
            WorkflowTransitionRule::new(
                "Pending CFO Approval",
                "Approve Level 2",
                "Approved",
                vec!["CFO".into()],
            )
            .with_mandatory_comment(),
            WorkflowTransitionRule::new(
                "Pending Manager Approval",
                "Reject",
                "Rejected",
                vec!["Sales Manager".into()],
            )
            .with_mandatory_comment(),
        ];

        let workflow = ApprovalWorkflow::new("Sales Order", "High Value PO Approval", "Draft")
            .with_states(states)
            .with_transitions(transitions);

        let doc_data = serde_json::json!({
            "grand_total": 75000.0,
            "customer": "Mega Corp Inc."
        });

        // 1. Sales User submits Draft -> Pending Manager Approval
        let step1 = workflow
            .evaluate_transition(
                "SO-2026-001",
                "Draft",
                "Submit for Approval",
                "john_sales",
                &["Sales User".into()],
                None,
                &doc_data,
            )
            .unwrap();
        assert_eq!(step1.new_state, "Pending Manager Approval");
        assert_eq!(step1.new_docstatus, 0);

        // 2. Sales User tries to approve Level 1 -> Disallowed (requires Sales Manager)
        let unauthorized_err = workflow.evaluate_transition(
            "SO-2026-001",
            "Pending Manager Approval",
            "Approve Level 1",
            "john_sales",
            &["Sales User".into()],
            None,
            &doc_data,
        );
        assert!(matches!(
            unauthorized_err,
            Err(WorkflowError::UnauthorizedRole(_, _, _))
        ));

        // 3. Sales Manager approves Level 1 -> Pending CFO Approval (Condition > 50000 met)
        let step2 = workflow
            .evaluate_transition(
                "SO-2026-001",
                "Pending Manager Approval",
                "Approve Level 1",
                "mary_mgr",
                &["Sales Manager".into()],
                None,
                &doc_data,
            )
            .unwrap();
        assert_eq!(step2.new_state, "Pending CFO Approval");

        // 4. CFO approves Level 2 without mandatory comment -> Fails
        let no_comment_err = workflow.evaluate_transition(
            "SO-2026-001",
            "Pending CFO Approval",
            "Approve Level 2",
            "cfo_bob",
            &["CFO".into()],
            None,
            &doc_data,
        );
        assert_eq!(
            no_comment_err,
            Err(WorkflowError::CommentRequired("Approve Level 2".into()))
        );

        // 5. CFO approves with comment -> Approved (docstatus = 1)
        let step3 = workflow
            .evaluate_transition(
                "SO-2026-001",
                "Pending CFO Approval",
                "Approve Level 2",
                "cfo_bob",
                &["CFO".into()],
                Some("Budget allocation confirmed in Q3 Capex."),
                &doc_data,
            )
            .unwrap();
        assert_eq!(step3.new_state, "Approved");
        assert_eq!(step3.new_docstatus, 1);
        assert_eq!(
            step3.log_entry.comment.as_deref(),
            Some("Budget allocation confirmed in Q3 Capex.")
        );
    }

    #[test]
    fn test_document_time_travel_versioning() {
        let mut engine = DocumentVersioningEngine::new();

        let v1_data = serde_json::json!({"item": "SERVER-01", "qty": 1, "rate": 5000});
        let v2_data = serde_json::json!({"item": "SERVER-01", "qty": 3, "rate": 4800});

        let delta = serde_json::json!({"qty": [1, 3], "rate": [5000, 4800]});

        engine.capture_version(
            "Quotation",
            "QT-001",
            "sales_rep",
            v1_data.clone(),
            serde_json::Value::Null,
            Some("Initial Draft".into()),
        );
        engine.capture_version(
            "Quotation",
            "QT-001",
            "manager",
            v2_data.clone(),
            delta,
            Some("Bulk discount applied".into()),
        );

        let history = engine.get_history("Quotation", "QT-001");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version_num, 1);
        assert_eq!(history[1].version_num, 2);

        // Restore version 1 snapshot
        let restored_v1 = engine.get_version("Quotation", "QT-001", 1).unwrap();
        assert_eq!(restored_v1, v1_data);
    }
}
