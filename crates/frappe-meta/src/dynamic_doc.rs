//! Dynamic Polymorphic Document Container & Zero-Allocation Document Bus (`frappe-meta::dynamic_doc`).
//!
//! Replaces interpreted Python dictionaries with a statically typed, stack-optimized
//! dynamic container using `CompactString` for field names and `SmallVec<[_; 16]>` for inline fields.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Strongly typed polymorphic field value within a dynamic document.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DocValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Currency(Decimal),
    Text(CompactString),
    Date(DateTime<Utc>),
    Array(Vec<DocValue>),
    Reference(CompactString), // Record ID pointer (e.g., "tab_item:item_001")
}

impl DocValue {
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DocValue::Text(s) | DocValue::Reference(s) => Some(s.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_currency(&self) -> Option<Decimal> {
        match self {
            DocValue::Currency(d) => Some(*d),
            DocValue::Int(i) => Some(Decimal::from(*i)),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            DocValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DocValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Zero-heap polymorphic document container.
/// Holds up to 16 dynamic fields entirely on the stack before spilling to the heap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DynamicDocument {
    pub doctype: CompactString,
    pub name: CompactString,
    /// Document status: 0 = Draft, 1 = Submitted, 2 = Cancelled
    pub docstatus: i8,
    pub owner: CompactString,
    pub creation: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub fields: SmallVec<[(CompactString, DocValue); 16]>,
}

impl DynamicDocument {
    /// Instantiates a new empty dynamic document with standard timestamps.
    #[must_use]
    pub fn new(
        doctype: impl Into<CompactString>,
        name: impl Into<CompactString>,
        owner: impl Into<CompactString>,
    ) -> Self {
        let now = Utc::now();
        Self {
            doctype: doctype.into(),
            name: name.into(),
            docstatus: 0,
            owner: owner.into(),
            creation: now,
            modified: now,
            fields: SmallVec::new(),
        }
    }

    /// Sets or updates a field value in the document.
    pub fn set_field(&mut self, key: impl Into<CompactString>, value: DocValue) {
        let k = key.into();
        for (field_name, field_val) in self.fields.iter_mut() {
            if field_name == &k {
                *field_val = value;
                return;
            }
        }
        self.fields.push((k, value));
    }

    /// Retrieves a field value reference with $O(N)$ lookup bounded at $N \le 16$.
    #[must_use]
    pub fn get_field(&self, key: &str) -> Option<&DocValue> {
        self.fields
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .map(|(_, v)| v)
    }

    /// Helper: returns string slice of a text field.
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get_field(key).and_then(DocValue::as_str)
    }

    /// Helper: returns Decimal currency value.
    #[must_use]
    pub fn get_currency(&self, key: &str) -> Option<Decimal> {
        self.get_field(key).and_then(DocValue::as_currency)
    }

    /// Helper: returns 64-bit integer value.
    #[must_use]
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_field(key).and_then(DocValue::as_int)
    }

    /// Helper: returns boolean value.
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_field(key).and_then(DocValue::as_bool)
    }

    /// Calculates estimated in-memory footprint in bytes.
    #[must_use]
    pub fn memory_footprint_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + if self.fields.spilled() {
                self.fields.capacity() * std::mem::size_of::<(CompactString, DocValue)>()
            } else {
                0
            }
    }
}

/// Dynamic SurrealQL DDL Generator for compiled SCHEMAFULL tables and field constraints.
pub struct SurrealDdlGenerator;

impl SurrealDdlGenerator {
    /// Compiles a DocType table schema and field definitions into SurrealQL DDL statements.
    #[must_use]
    pub fn generate_table_ddl(table_name: &str, fields: &[(&str, &str, bool)]) -> String {
        let mut ddl = format!("DEFINE TABLE {} SCHEMAFULL;\n", table_name);

        for (field_name, field_type, is_required) in fields {
            let surreal_type = match *field_type {
                "Currency" => "decimal ASSERT $value >= 0",
                "Int" => "int",
                "Float" => "float",
                "Bool" => "bool",
                "Date" | "DateTime" => "datetime",
                "Link" => "record",
                _ => "string",
            };

            ddl.push_str(&format!(
                "DEFINE FIELD {} ON TABLE {} TYPE {};\n",
                field_name, table_name, surreal_type
            ));

            if *is_required {
                ddl.push_str(&format!(
                    "DEFINE FIELD {} ON TABLE {} ASSERT $value != NONE;\n",
                    field_name, table_name
                ));
            }
        }

        ddl
    }

    /// Generates a SurrealQL RELATE graph edge statement for child table linking.
    #[must_use]
    pub fn generate_child_relate(
        parent_table: &str,
        parent_id: &str,
        child_table: &str,
        child_id: &str,
        order_idx: usize,
    ) -> String {
        format!(
            "RELATE {}:{}->has_child->{}:{} SET order_idx = {};",
            parent_table, parent_id, child_table, child_id, order_idx
        )
    }

    /// Generates a SurrealQL query selecting parent document with all traversed child items.
    #[must_use]
    pub fn generate_child_query(parent_table: &str, parent_id: &str, child_table: &str) -> String {
        format!(
            "SELECT *, ->has_child->({} AS items) FROM {}:{};",
            child_table, parent_table, parent_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_dynamic_document_stack_storage() {
        let mut doc = DynamicDocument::new("SalesInvoice", "INV-2026-0001", "administrator");

        assert_eq!(doc.doctype.as_str(), "SalesInvoice");
        assert_eq!(doc.name.as_str(), "INV-2026-0001");
        assert_eq!(doc.docstatus, 0);

        // Populate fields
        doc.set_field("customer", DocValue::Text("ACME Corp".into()));
        doc.set_field("grand_total", DocValue::Currency(dec!(4999.50)));
        doc.set_field("item_count", DocValue::Int(5));
        doc.set_field("is_paid", DocValue::Bool(false));

        assert_eq!(doc.get_str("customer"), Some("ACME Corp"));
        assert_eq!(doc.get_currency("grand_total"), Some(dec!(4999.50)));
        assert_eq!(doc.get_int("item_count"), Some(5));
        assert_eq!(doc.get_bool("is_paid"), Some(false));

        // Ensure small vector does not spill for <= 16 fields
        assert!(!doc.fields.spilled());
    }

    #[test]
    fn test_dynamic_document_json_roundtrip() {
        let mut doc = DynamicDocument::new("Customer", "CUST-001", "system");
        doc.set_field("company_name", DocValue::Text("Apex Solutions".into()));
        doc.set_field("credit_limit", DocValue::Currency(dec!(50000.00)));

        let json = serde_json::to_string(&doc).unwrap();
        let decoded: DynamicDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.doctype, doc.doctype);
        assert_eq!(decoded.get_str("company_name"), Some("Apex Solutions"));
        assert_eq!(decoded.get_currency("credit_limit"), Some(dec!(50000.00)));
    }

    #[test]
    fn test_surreal_ddl_generation() {
        let fields = vec![
            ("customer", "Text", true),
            ("grand_total", "Currency", true),
            ("item_count", "Int", false),
        ];

        let ddl = SurrealDdlGenerator::generate_table_ddl("tab_sales_invoice", &fields);
        assert!(ddl.contains("DEFINE TABLE tab_sales_invoice SCHEMAFULL;"));
        assert!(ddl.contains(
            "DEFINE FIELD grand_total ON TABLE tab_sales_invoice TYPE decimal ASSERT $value >= 0;"
        ));
        assert!(
            ddl.contains("DEFINE FIELD customer ON TABLE tab_sales_invoice ASSERT $value != NONE;")
        );
    }

    #[test]
    fn test_surreal_relate_generation() {
        let relate_sql = SurrealDdlGenerator::generate_child_relate(
            "sales_order",
            "SO_001",
            "sales_order_item",
            "SOI_001",
            0,
        );
        assert_eq!(
            relate_sql,
            "RELATE sales_order:SO_001->has_child->sales_order_item:SOI_001 SET order_idx = 0;"
        );

        let query_sql = SurrealDdlGenerator::generate_child_query(
            "sales_order",
            "SO_001",
            "sales_order_item",
        );
        assert_eq!(
            query_sql,
            "SELECT *, ->has_child->(sales_order_item AS items) FROM sales_order:SO_001;"
        );
    }
}
