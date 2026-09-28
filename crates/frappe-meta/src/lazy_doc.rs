//! Lazy Document Materialization & Virtual Child Table Proxy (`frappe-meta::lazy_doc`).
//!
//! Defer child table deserialization and foreign entity link joins until fields
//! are explicitly accessed in business logic (`frappe.get_lazy_doc()`).
//! Provides virtual child tables as in-memory views without physical storage writes unless mutated.

use crate::dynamic_doc::{DocValue, DynamicDocument};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual Child Table Row representing an in-memory view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VirtualChildRow {
    pub name: CompactString,
    pub parent: CompactString,
    pub parenttype: CompactString,
    pub parentfield: CompactString,
    pub idx: u32,
    pub is_dirty: bool,
    pub fields: HashMap<CompactString, DocValue>,
}

impl VirtualChildRow {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        parent: impl Into<CompactString>,
        parenttype: impl Into<CompactString>,
        parentfield: impl Into<CompactString>,
        idx: u32,
    ) -> Self {
        Self {
            name: name.into(),
            parent: parent.into(),
            parenttype: parenttype.into(),
            parentfield: parentfield.into(),
            idx,
            is_dirty: false,
            fields: HashMap::new(),
        }
    }

    pub fn set_field(&mut self, key: impl Into<CompactString>, value: DocValue) {
        self.fields.insert(key.into(), value);
        self.is_dirty = true;
    }

    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&DocValue> {
        self.fields.get(key)
    }
}

/// Lazy Document proxy wrapping a primary DynamicDocument and deferring child/relational lookups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LazyDocument {
    pub base_doc: DynamicDocument,
    pub is_materialized: bool,
    pub virtual_tables: HashMap<CompactString, Vec<VirtualChildRow>>,
    pub link_cache: HashMap<CompactString, DynamicDocument>,
}

impl LazyDocument {
    #[must_use]
    pub fn new(base_doc: DynamicDocument) -> Self {
        Self {
            base_doc,
            is_materialized: false,
            virtual_tables: HashMap::new(),
            link_cache: HashMap::new(),
        }
    }

    /// Accesses a base document field directly without triggering child materialization.
    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&DocValue> {
        self.base_doc.get_field(key)
    }

    /// Lazily mounts a virtual child table view.
    pub fn mount_virtual_table(
        &mut self,
        table_field: impl Into<CompactString>,
        rows: Vec<VirtualChildRow>,
    ) {
        self.virtual_tables.insert(table_field.into(), rows);
    }

    /// Materializes child tables on explicit request.
    pub fn materialize_table(&mut self, table_field: &str) -> Option<&mut Vec<VirtualChildRow>> {
        self.is_materialized = true;
        self.virtual_tables.get_mut(table_field)
    }

    /// Checks if any virtual child row has been mutated, requiring physical persistence.
    #[must_use]
    pub fn has_dirty_virtual_rows(&self) -> bool {
        self.virtual_tables
            .values()
            .any(|rows| rows.iter().any(|r| r.is_dirty))
    }

    /// Retrieves dirty rows that need physical database storage writes.
    #[must_use]
    pub fn get_dirty_virtual_rows(&self) -> Vec<&VirtualChildRow> {
        self.virtual_tables
            .values()
            .flat_map(|rows| rows.iter().filter(|r| r.is_dirty))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_lazy_doc_and_virtual_child_tables() {
        let base = DynamicDocument::new("SalesInvoice", "SINV-2026-0010", "admin");
        let mut lazy = LazyDocument::new(base);

        assert!(!lazy.is_materialized);
        assert!(!lazy.has_dirty_virtual_rows());

        let mut row1 = VirtualChildRow::new("row-1", "SINV-2026-0010", "SalesInvoice", "items", 1);
        row1.set_field("item_code", DocValue::Text("ITEM-A".into()));
        row1.set_field("rate", DocValue::Currency(dec!(100.00)));

        let mut row2 = VirtualChildRow::new("row-2", "SINV-2026-0010", "SalesInvoice", "items", 2);
        row2.fields.insert("item_code".into(), DocValue::Text("ITEM-B".into()));
        row2.is_dirty = false; // unmodified view

        lazy.mount_virtual_table("items", vec![row1, row2]);

        assert!(lazy.has_dirty_virtual_rows());
        let dirty = lazy.get_dirty_virtual_rows();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].name.as_str(), "row-1");
    }
}
