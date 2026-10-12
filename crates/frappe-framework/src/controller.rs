//! Document Controller Trait & Typed Lifecycle Pipeline (`frappe-framework::controller`).
//!
//! Provides the strongly typed Rust controller interface mirroring Frappe's
//! `frappe.model.document.Document` controller classes. Enables native, zero-cost,
//! memory-safe business logic execution during document lifecycle transitions.

use crate::lifecycle::{Document, DocumentError};
use compact_str::CompactString;
use frappe_meta::DocTypeSchema;
use std::collections::HashMap;
use std::sync::Arc;

/// Strongly typed Rust controller interface mirroring Frappe's `frappe.model.document.Document`.
pub trait DocController: Send + Sync {
    /// Hook executed before document validation.
    fn before_validate(
        &self,
        _doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed to validate business invariants, field links, and calculation rules.
    fn validate(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed before saving document to persistence.
    fn before_save(
        &self,
        _doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed after document persistence commit.
    fn after_save(
        &self,
        _doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed before document submission.
    fn before_submit(
        &self,
        _doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed on document submission (e.g. posting General Ledger or Stock Ledger entries).
    fn on_submit(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed before document cancellation.
    fn before_cancel(
        &self,
        _doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed on document cancellation (e.g. reversing ledger entries).
    fn on_cancel(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        Ok(())
    }

    /// Hook executed before document deletion / trashing.
    fn on_trash(&self, _doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        Ok(())
    }
}

/// Registry mapping DocType names to their respective typed Rust controllers.
#[derive(Default, Clone)]
pub struct ControllerRegistry {
    controllers: HashMap<CompactString, Arc<dyn DocController>>,
}

impl ControllerRegistry {
    /// Creates a new empty controller registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a typed controller for a specific DocType.
    pub fn register<C>(&mut self, doctype: &str, controller: C)
    where
        C: DocController + 'static,
    {
        self.controllers
            .insert(CompactString::new(doctype), Arc::new(controller));
    }

    /// Registers a boxed controller for a specific DocType.
    pub fn register_arc(&mut self, doctype: &str, controller: Arc<dyn DocController>) {
        self.controllers
            .insert(CompactString::new(doctype), controller);
    }

    /// Looks up the controller registered for the given DocType.
    #[must_use]
    pub fn get(&self, doctype: &str) -> Option<Arc<dyn DocController>> {
        self.controllers.get(doctype).cloned()
    }

    /// Returns the number of registered controllers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.controllers.len()
    }

    /// Returns true if no controllers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestInvoiceController;

    impl DocController for TestInvoiceController {
        fn validate(
            &self,
            doc: &mut Document,
            _schema: &DocTypeSchema,
        ) -> Result<(), DocumentError> {
            if let Some(obj) = doc.data.as_object_mut() {
                obj.insert("validated".into(), serde_json::json!(true));
            }
            Ok(())
        }
    }

    #[test]
    fn test_controller_registry() {
        let mut registry = ControllerRegistry::new();
        registry.register("Sales Invoice", TestInvoiceController);

        let controller = registry
            .get("Sales Invoice")
            .expect("Controller must exist");
        let mut doc = Document::new("Sales Invoice", serde_json::json!({ "total": 100 }));
        let schema = DocTypeSchema {
            name: "Sales Invoice".into(),
            module: "Accounts".into(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            fields: vec![],
            permissions: vec![],
        };

        controller.validate(&mut doc, &schema).unwrap();
        assert_eq!(doc.data["validated"], serde_json::json!(true));
    }
}
