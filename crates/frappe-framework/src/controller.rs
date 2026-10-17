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

    /// Pre-populates the registry with the complete suite of canonical upstream submittable document controllers (all 86) plus core business controllers.
    #[must_use]
    pub fn with_standard_submittables() -> Self {
        let mut reg = Self::new();
        for &doctype in ALL_SUBMITTABLE_DOCTYPES {
            reg.register(doctype, StandardSubmittableController);
        }
        reg.register("Sales Invoice", SalesInvoiceController);
        reg.register("Purchase Invoice", PurchaseInvoiceController);
        reg.register("Payment Entry", PaymentEntryController);
        reg.register("Journal Entry", JournalEntryController);
        reg.register("Stock Entry", StockEntryController);
        reg.register("Delivery Note", DeliveryNoteController);
        reg.register("Purchase Receipt", PurchaseReceiptController);
        reg
    }
}

/// Standard canonical controller for all submittable upstream DocTypes.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardSubmittableController;

impl DocController for StandardSubmittableController {
    fn before_submit(
        &self,
        _doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }
        Ok(())
    }

    fn on_submit(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 1;
        Ok(())
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }
        Ok(())
    }

    fn on_cancel(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 2;
        Ok(())
    }
}

/// Typed controller for Sales Invoice.
#[derive(Debug, Clone, Copy, Default)]
pub struct SalesInvoiceController;

impl DocController for SalesInvoiceController {
    fn validate(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        if let Some(items) = doc.data.get("items").and_then(|v| v.as_array())
            && items.is_empty()
        {
            return Err(DocumentError::ValidationFailed(
                "Sales Invoice must have at least one line item".to_string(),
            ));
        }
        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Purchase Invoice.
#[derive(Debug, Clone, Copy, Default)]
pub struct PurchaseInvoiceController;

impl DocController for PurchaseInvoiceController {
    fn validate(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        if let Some(items) = doc.data.get("items").and_then(|v| v.as_array())
            && items.is_empty()
        {
            return Err(DocumentError::ValidationFailed(
                "Purchase Invoice must have at least one line item".to_string(),
            ));
        }
        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Payment Entry.
#[derive(Debug, Clone, Copy, Default)]
pub struct PaymentEntryController;

impl DocController for PaymentEntryController {
    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Journal Entry.
#[derive(Debug, Clone, Copy, Default)]
pub struct JournalEntryController;

impl DocController for JournalEntryController {
    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Stock Entry.
#[derive(Debug, Clone, Copy, Default)]
pub struct StockEntryController;

impl DocController for StockEntryController {
    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Delivery Note.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeliveryNoteController;

impl DocController for DeliveryNoteController {
    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Typed controller for Purchase Receipt.
#[derive(Debug, Clone, Copy, Default)]
pub struct PurchaseReceiptController;

impl DocController for PurchaseReceiptController {
    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_submit(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_submit(doc, schema)
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        StandardSubmittableController.before_cancel(doc, schema)
    }

    fn on_cancel(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        StandardSubmittableController.on_cancel(doc, schema)
    }
}

/// Canonical array of all 86 upstream submittable DocTypes.
pub const ALL_SUBMITTABLE_DOCTYPES: &[&str] = &[
    "Account Closing Balance",
    "Advance Payment Ledger Entry",
    "Asset",
    "Asset Capitalization",
    "Asset Depreciation Schedule",
    "Asset Maintenance Log",
    "Asset Movement",
    "Asset Repair",
    "Asset Shift Allocation",
    "Asset Value Adjustment",
    "BOM",
    "BOM Creator",
    "BOM Update Log",
    "Bank Guarantee",
    "Bank Transaction",
    "Blanket Order",
    "Budget",
    "Cashier Closing",
    "Contract",
    "Contract Fulfilment Checklist",
    "Cost Center Allocation",
    "Delivery Note",
    "Delivery Trip",
    "DuckDB Sync",
    "Dunning",
    "Exchange Rate Revaluation",
    "GL Entry",
    "Installation Note",
    "Invoice Discounting",
    "Item Standard Cost",
    "Job Card",
    "Journal Entry",
    "Landed Cost Voucher",
    "Maintenance Schedule",
    "Maintenance Visit",
    "MapReduce Job",
    "Material Request",
    "POS Closing Entry",
    "POS Invoice",
    "POS Invoice Merge Log",
    "POS Opening Entry",
    "Packing Slip",
    "Payment Entry",
    "Payment Ledger Entry",
    "Payment Order",
    "Payment Request",
    "Period Closing Voucher",
    "Pick List",
    "Process Deferred Accounting",
    "Process Payment Reconciliation",
    "Process Period Closing Voucher",
    "Process Subscription",
    "Product Bundle",
    "Production Plan",
    "Proforma Invoice",
    "Project Update",
    "Purchase Invoice",
    "Purchase Order",
    "Purchase Receipt",
    "Quality Inspection",
    "Quotation",
    "Repost Accounting Ledger",
    "Repost Item Valuation",
    "Repost Payment Ledger",
    "Request for Quotation",
    "Sales Forecast",
    "Sales Invoice",
    "Sales Order",
    "Serial and Batch Bundle",
    "Share Transfer",
    "Shipment",
    "Stock Closing Entry",
    "Stock Entry",
    "Stock Ledger Entry",
    "Stock Reconciliation",
    "Stock Reservation Entry",
    "Subcontracting Inward Order",
    "Subcontracting Order",
    "Subcontracting Receipt",
    "Supplier Quotation",
    "Supplier Scorecard Period",
    "Telephony Call Type",
    "Timesheet",
    "Transaction Deletion Record",
    "Unreconcile Payment",
    "Work Order",
];

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
            title_field: None,
            search_fields: None,
            image_field: None,
            fields: vec![],
            permissions: vec![],
        };

        controller.validate(&mut doc, &schema).unwrap();
        assert_eq!(doc.data["validated"], serde_json::json!(true));
    }

    #[test]
    fn test_all_86_submittable_controllers() {
        let registry = ControllerRegistry::with_standard_submittables();
        assert_eq!(registry.len(), 86);
        assert_eq!(ALL_SUBMITTABLE_DOCTYPES.len(), 86);

        for &dt in ALL_SUBMITTABLE_DOCTYPES {
            let ctrl = registry
                .get(dt)
                .unwrap_or_else(|| panic!("Submittable controller for '{dt}' missing"));

            let mut doc = Document::new(dt, serde_json::json!({ "name": "TEST-001" }));
            let schema = DocTypeSchema {
                name: dt.into(),
                module: "Test".into(),
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
                title_field: None,
                search_fields: None,
                image_field: None,
                fields: vec![],
                permissions: vec![],
            };

            // Test Submit Transition
            ctrl.before_submit(&mut doc, &schema).unwrap();
            ctrl.on_submit(&mut doc, &schema).unwrap();
            assert_eq!(doc.docstatus, 1);

            // Test Cancel Transition
            ctrl.before_cancel(&mut doc, &schema).unwrap();
            ctrl.on_cancel(&mut doc, &schema).unwrap();
            assert_eq!(doc.docstatus, 2);
        }
    }
}
