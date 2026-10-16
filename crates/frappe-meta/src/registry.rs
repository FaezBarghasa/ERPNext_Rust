//! Canonical DocType Schema Registry & Scaffold Engine (`frappe-meta::registry`).
//!
//! Provides the central registry for standard ERPNext and custom DocTypes,
//! automatic validation, and batch SurrealQL DDL compilation.

use crate::generated::v16_schemas;
use crate::schema::{DocTypeSchema, SchemaError};
use crate::schema_compiler::compile_to_surrealql;
use compact_str::CompactString;
use std::collections::HashMap;

/// Central in-memory registry of validated DocType schemas.
#[derive(Debug, Clone, Default)]
pub struct DocTypeRegistry {
    doctypes: HashMap<CompactString, DocTypeSchema>,
}

impl DocTypeRegistry {
    /// Creates a new empty DocType registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a registry pre-populated with standard ERPNext business DocTypes.
    #[must_use]
    pub fn new_standard_erpnext() -> Self {
        let mut registry = Self::new();
        for schema in standard_erpnext_schemas() {
            let _ = registry.register(schema);
        }
        registry
    }

    /// Initializes a complete registry pre-populated with all 840+ ERPNext v16 & Frappe v16 DocTypes.
    #[must_use]
    pub fn new_v16_full() -> Self {
        let mut registry = Self::new();
        for schema in v16_schemas::all_v16_schemas() {
            let _ = registry.register(schema);
        }
        registry
    }

    /// Registers a new DocType schema into the registry, validating its fields and rules.
    pub fn register(&mut self, schema: DocTypeSchema) -> Result<(), SchemaError> {
        schema.validate()?;
        let key = CompactString::new(&schema.name);
        self.doctypes.insert(key, schema);
        Ok(())
    }

    /// Retrieves a reference to a registered DocType schema by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&DocTypeSchema> {
        self.doctypes.get(name)
    }

    /// Returns the number of registered DocTypes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.doctypes.len()
    }

    /// Returns true if the registry contains no DocTypes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.doctypes.is_empty()
    }

    /// Lists all registered DocType names sorted alphabetically.
    #[must_use]
    pub fn list_doctypes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.doctypes.keys().map(|k| k.to_string()).collect();
        names.sort();
        names
    }

    /// Compiles all registered DocType schemas into a unified sequence of SurrealQL statements.
    pub fn compile_all_surrealql(&self) -> Result<Vec<String>, SchemaError> {
        let mut ddl = Vec::new();
        for schema in self.doctypes.values() {
            let statements = compile_to_surrealql(schema)?;
            ddl.extend(statements);
        }
        Ok(ddl)
    }
}

/// Constructs the canonical list of standard ERPNext DocType schemas generated from v16 definitions.
#[must_use]
pub fn standard_erpnext_schemas() -> Vec<DocTypeSchema> {
    vec![
        // Core Setup
        v16_schemas::schema_company(),
        v16_schemas::schema_user(),
        v16_schemas::schema_role(),
        // Accounts Module
        v16_schemas::schema_account(),
        v16_schemas::schema_gl_entry(),
        v16_schemas::schema_journal_entry(),
        v16_schemas::schema_sales_invoice(),
        v16_schemas::schema_purchase_invoice(),
        v16_schemas::schema_payment_entry(),
        // Stock / Inventory Module
        v16_schemas::schema_item(),
        v16_schemas::schema_item_group(),
        v16_schemas::schema_warehouse(),
        v16_schemas::schema_stock_entry(),
        v16_schemas::schema_stock_ledger_entry(),
        v16_schemas::schema_batch(),
        v16_schemas::schema_serial_no(),
        // Selling Module
        v16_schemas::schema_customer(),
        v16_schemas::schema_quotation(),
        v16_schemas::schema_sales_order(),
        v16_schemas::schema_delivery_note(),
        // Buying Module
        v16_schemas::schema_supplier(),
        v16_schemas::schema_purchase_order(),
        v16_schemas::schema_purchase_receipt(),
        // Manufacturing Module
        v16_schemas::schema_bom(),
        v16_schemas::schema_work_order(),
        v16_schemas::schema_operation(),
        v16_schemas::schema_workstation(),
        // Projects Module
        v16_schemas::schema_project(),
        v16_schemas::schema_task(),
        v16_schemas::schema_timesheet(),
        // CRM Module
        v16_schemas::schema_lead(),
        v16_schemas::schema_opportunity(),
        // Assets Module
        v16_schemas::schema_asset(),
        v16_schemas::schema_asset_category(),
        // Quality Module
        v16_schemas::schema_quality_inspection(),
        v16_schemas::schema_quality_inspection_template(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_erpnext_registry() {
        let registry = DocTypeRegistry::new_standard_erpnext();
        assert!(registry.len() >= 25);
        assert!(registry.get("Company").is_some());
        assert!(registry.get("Sales Invoice").is_some());
        assert!(registry.get("Item").is_some());
        assert!(registry.get("Stock Entry").is_some());
        assert!(registry.get("Purchase Order").is_some());
        assert!(registry.get("BOM").is_some());
        assert!(registry.get("Project").is_some());

        // Validate that all standard schemas compile to valid SurrealQL DDL
        let ddl = registry
            .compile_all_surrealql()
            .expect("DDL compilation must succeed");
        assert!(!ddl.is_empty());
    }

    #[test]
    fn test_v16_full_registry() {
        let registry = DocTypeRegistry::new_v16_full();
        assert!(registry.len() >= 800);
        assert!(registry.get("Sales Invoice").is_some());
        assert!(registry.get("Purchase Invoice").is_some());
        assert!(registry.get("Payment Entry").is_some());
        assert!(registry.get("General Ledger").is_some() || registry.get("GL Entry").is_some());
        assert!(registry.get("Stock Entry").is_some());
        assert!(registry.get("Work Order").is_some());
        assert!(registry.get("Quality Inspection").is_some());
    }
}
