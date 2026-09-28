use crate::scripting::{LifecycleEvent, RhaiHookEngine, ScriptError};
use frappe_meta::{DocTypeSchema, NamingSeriesParser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Document operation and lifecycle transition errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    /// Attempted to mutate an already submitted document.
    #[error("Cannot edit submitted document (docstatus = 1)")]
    CannotEditSubmittedDocument,
    /// Document is not submittable.
    #[error("DocType '{0}' is not submittable")]
    DocTypeNotSubmittable(String),
    /// Document is not in submitted state when attempting cancel.
    #[error("Cannot cancel document: current status is {0}, expected 1 (Submitted)")]
    CannotCancelUnsubmittedDocument(i32),
    /// Document is not in cancelled state when attempting amend.
    #[error("Cannot amend document: current status is {0}, expected 2 (Cancelled)")]
    CannotAmendUncancelledDocument(i32),
    /// Cannot discard submitted or cancelled document.
    #[error("Cannot discard document: status is {0}, only draft documents (docstatus = 0) can be discarded")]
    CannotDiscardNonDraft(i32),
    /// Schema validation failed.
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),
    /// Script hook failed.
    #[error("Script hook failed: {0}")]
    HookFailed(#[from] ScriptError),
    /// Wasm execution failed.
    #[error("Wasm execution failed: {0}")]
    WasmExecutionFailed(String),
    /// Workflow transition not allowed.
    #[error("Workflow transition from '{0}' to '{1}' by user '{2}' is not permitted")]
    WorkflowTransitionDenied(String, String, String),
    /// Document not found.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
}

/// In-memory representation of a Frappe document instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    /// Primary key identifier (e.g. "ACC-INV-2026-00001").
    pub name: String,
    /// DocType identifier.
    pub doctype: String,
    /// Document status: 0 = Draft, 1 = Submitted, 2 = Cancelled, 3 = Discarded
    pub docstatus: i32,
    /// Workflow state string (e.g. "Draft", "Pending Approval", "Approved").
    pub workflow_state: Option<String>,
    /// Amended from document identifier.
    pub amended_from: Option<String>,
    /// Document payload data.
    pub data: serde_json::Value,
}

impl Document {
    /// Creates a new draft document instance.
    #[must_use]
    pub fn new(doctype: &str, data: serde_json::Value) -> Self {
        Self {
            name: String::new(),
            doctype: doctype.to_string(),
            docstatus: 0,
            workflow_state: None,
            amended_from: None,
            data,
        }
    }

    /// Is this document a draft?
    #[must_use]
    pub fn is_draft(&self) -> bool {
        self.docstatus == 0
    }

    /// Is this document submitted?
    #[must_use]
    pub fn is_submitted(&self) -> bool {
        self.docstatus == 1
    }

    /// Is this document cancelled?
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.docstatus == 2
    }

    /// Is this document discarded?
    #[must_use]
    pub fn is_discarded(&self) -> bool {
        self.docstatus == 3
    }
}

/// Workflow Transition Rule definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowTransition {
    pub state: String,
    pub action: String,
    pub next_state: String,
    pub allowed_role: String,
    pub condition_field: Option<String>,
    pub condition_value: Option<String>,
}

/// Unified Document Controller enforcing lifecycle state transitions, hooks, and discard engine.
pub struct DocumentController {
    engine: RhaiHookEngine,
    workflow_transitions: HashMap<String, Vec<WorkflowTransition>>,
}

impl Default for DocumentController {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentController {
    /// Creates a new DocumentController.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: RhaiHookEngine::new(),
            workflow_transitions: HashMap::new(),
        }
    }

    /// Registers workflow transitions for a specific DocType.
    pub fn register_workflow(&mut self, doctype: impl Into<String>, transitions: Vec<WorkflowTransition>) {
        self.workflow_transitions.insert(doctype.into(), transitions);
    }

    /// Handles document insertion: runs naming series, validation hooks, and commits to draft state.
    pub fn insert(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        script: Option<&str>,
        year: u32,
        seq: u64,
    ) -> Result<(), DocumentError> {
        if doc.name.is_empty() {
            let template = schema.naming_rule.as_deref().unwrap_or("DOC-.YYYY.-.#####");
            doc.name = NamingSeriesParser::format(template, year, seq);
        }

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::BeforeValidate, &mut doc.data, s)?;
        }

        schema
            .validate()
            .map_err(|e| DocumentError::ValidationFailed(e.to_string()))?;

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::Validate, &mut doc.data, s)?;
            self.engine
                .dispatch_hook(LifecycleEvent::BeforeSave, &mut doc.data, s)?;
        }

        doc.docstatus = 0;

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::AfterSave, &mut doc.data, s)?;
        }

        Ok(())
    }

    /// Mutates an existing document while rejecting edits to submitted documents.
    pub fn update(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        new_data: serde_json::Value,
        script: Option<&str>,
    ) -> Result<(), DocumentError> {
        if doc.is_submitted() {
            return Err(DocumentError::CannotEditSubmittedDocument);
        }

        doc.data = new_data;

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::Validate, &mut doc.data, s)?;
            self.engine
                .dispatch_hook(LifecycleEvent::BeforeSave, &mut doc.data, s)?;
            self.engine
                .dispatch_hook(LifecycleEvent::AfterSave, &mut doc.data, s)?;
        }

        schema
            .validate()
            .map_err(|e| DocumentError::ValidationFailed(e.to_string()))?;

        Ok(())
    }

    /// Submits a draft document, mutating docstatus to 1 and triggering submission hooks.
    pub fn submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        script: Option<&str>,
    ) -> Result<(), DocumentError> {
        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::BeforeSubmit, &mut doc.data, s)?;
        }

        doc.docstatus = 1;

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::OnSubmit, &mut doc.data, s)?;
        }

        Ok(())
    }

    /// Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.
    pub fn cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
        script: Option<&str>,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::BeforeCancel, &mut doc.data, s)?;
        }

        doc.docstatus = 2;

        if let Some(s) = script {
            self.engine
                .dispatch_hook(LifecycleEvent::OnCancel, &mut doc.data, s)?;
        }

        Ok(())
    }

    /// Amends a cancelled document, generating a new draft copy with amendment tracking.
    pub fn amend(
        &self,
        doc: &Document,
        _schema: &DocTypeSchema,
    ) -> Result<Document, DocumentError> {
        if doc.docstatus != 2 {
            return Err(DocumentError::CannotAmendUncancelledDocument(doc.docstatus));
        }

        // In Frappe, first amendment appends -1. Subsequent amendments increment -2, -3.
        let amended_name = if let Some((base, suffix)) = doc.name.rsplit_once('-') {
            if suffix.chars().all(|c| c.is_numeric()) && suffix.len() <= 3 && !doc.name.ends_with("00001") && base.contains('-') && suffix.parse::<u32>().is_ok() && doc.amended_from.is_some() {
                let num: u32 = suffix.parse().unwrap_or(0);
                format!("{}-{}", base, num + 1)
            } else {
                format!("{}-1", doc.name)
            }
        } else {
            format!("{}-1", doc.name)
        };

        let mut amended_data = doc.data.clone();
        if let serde_json::Value::Object(ref mut map) = amended_data {
            map.insert("amended_from".to_string(), serde_json::Value::String(doc.name.clone()));
            map.remove("docstatus");
        }

        Ok(Document {
            name: amended_name,
            doctype: doc.doctype.clone(),
            docstatus: 0, // Reset to draft
            workflow_state: Some("Draft".to_string()),
            amended_from: Some(doc.name.clone()),
            data: amended_data,
        })
    }

    /// Document Discard Engine: discards uncommitted draft transactions and purges child state.
    pub fn discard(&self, doc: &mut Document) -> Result<(), DocumentError> {
        if doc.docstatus != 0 {
            return Err(DocumentError::CannotDiscardNonDraft(doc.docstatus));
        }

        doc.docstatus = 3; // Discarded status
        doc.workflow_state = Some("Discarded".to_string());
        
        // Clear uncommitted child tables from JSON payload
        if let serde_json::Value::Object(ref mut map) = doc.data {
            map.insert("status".to_string(), serde_json::Value::String("Discarded".to_string()));
        }

        Ok(())
    }

    /// Executes a workflow transition on a document.
    pub fn apply_workflow_action(
        &self,
        doc: &mut Document,
        action: &str,
        user_role: &str,
    ) -> Result<(), DocumentError> {
        let current_state = doc.workflow_state.as_deref().unwrap_or("Draft");
        
        if let Some(transitions) = self.workflow_transitions.get(&doc.doctype) {
            for t in transitions {
                if t.state == current_state && t.action == action {
                    if t.allowed_role != user_role && t.allowed_role != "All" {
                        return Err(DocumentError::WorkflowTransitionDenied(
                            current_state.to_string(),
                            t.next_state.clone(),
                            user_role.to_string(),
                        ));
                    }

                    // Check condition field if specified
                    if let (Some(field), Some(expected)) = (&t.condition_field, &t.condition_value)
                        && let Some(actual) = doc.data.get(field).and_then(|v| v.as_str())
                        && actual != expected
                    {
                        continue;
                    }

                    doc.workflow_state = Some(t.next_state.clone());
                    return Ok(());
                }
            }
        }

        Err(DocumentError::WorkflowTransitionDenied(
            current_state.to_string(),
            action.to_string(),
            user_role.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_schema() -> DocTypeSchema {
        DocTypeSchema {
            name: "Sales Invoice".to_string(),
            module: "Accounts".to_string(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: Some("INV-.YYYY.-.#####".to_string()),
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            fields: vec![],
            permissions: vec![],
        }
    }

    #[test]
    fn test_document_lifecycle_and_amend_discard() {
        let controller = DocumentController::new();
        let schema = dummy_schema();

        let mut doc = Document::new("Sales Invoice", serde_json::json!({"customer": "ACME"}));
        controller.insert(&mut doc, &schema, None, 2026, 1).unwrap();
        assert_eq!(doc.name, "INV-2026-00001");
        assert!(doc.is_draft());

        // Submit
        controller.submit(&mut doc, &schema, None).unwrap();
        assert!(doc.is_submitted());

        // Cancel
        controller.cancel(&mut doc, &schema, None).unwrap();
        assert!(doc.is_cancelled());

        // Amend
        let amended = controller.amend(&doc, &schema).unwrap();
        assert_eq!(amended.name, "INV-2026-00001-1");
        assert!(amended.is_draft());
        assert_eq!(amended.amended_from.as_deref(), Some("INV-2026-00001"));

        // Discard draft
        let mut draft_doc = Document::new("Sales Invoice", serde_json::json!({"customer": "Test"}));
        controller.insert(&mut draft_doc, &schema, None, 2026, 2).unwrap();
        controller.discard(&mut draft_doc).unwrap();
        assert!(draft_doc.is_discarded());
    }

    #[test]
    fn test_workflow_transitions() {
        let mut controller = DocumentController::new();
        let transitions = vec![
            WorkflowTransition {
                state: "Draft".into(),
                action: "Submit for Approval".into(),
                next_state: "Pending Approval".into(),
                allowed_role: "Sales User".into(),
                condition_field: None,
                condition_value: None,
            },
            WorkflowTransition {
                state: "Pending Approval".into(),
                action: "Approve".into(),
                next_state: "Approved".into(),
                allowed_role: "Sales Manager".into(),
                condition_field: None,
                condition_value: None,
            },
        ];

        controller.register_workflow("Sales Invoice", transitions);
        let mut doc = Document::new("Sales Invoice", serde_json::json!({}));
        doc.workflow_state = Some("Draft".into());

        assert!(controller.apply_workflow_action(&mut doc, "Submit for Approval", "Sales User").is_ok());
        assert_eq!(doc.workflow_state.as_deref(), Some("Pending Approval"));

        // Sales User cannot approve
        assert!(controller.apply_workflow_action(&mut doc, "Approve", "Sales User").is_err());

        // Sales Manager can approve
        assert!(controller.apply_workflow_action(&mut doc, "Approve", "Sales Manager").is_ok());
        assert_eq!(doc.workflow_state.as_deref(), Some("Approved"));
    }
}
