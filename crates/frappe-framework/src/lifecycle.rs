use crate::scripting::{LifecycleEvent, RhaiHookEngine, ScriptError};
use frappe_meta::{DocTypeSchema, NamingSeriesParser};
use serde::{Deserialize, Serialize};
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
    /// Schema validation failed.
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),
    /// Script hook failed.
    #[error("Script hook failed: {0}")]
    HookFailed(#[from] ScriptError),
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
    /// Document status: 0 = Draft, 1 = Submitted, 2 = Cancelled.
    pub docstatus: i32,
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
}

/// Unified Document Controller enforcing lifecycle state transitions and hooks.
pub struct DocumentController {
    engine: RhaiHookEngine,
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
        }
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
}
