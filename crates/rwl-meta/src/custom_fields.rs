//! Custom Fields and Dynamic Runtime Extensibility (`frappe-meta::custom_fields`).
//!
//! Provides Frappe-equivalent dynamic custom fields, custom DocTypes, and schema augmentation:
//! - `CustomField`: Definition for extending standard DocTypes without modifying core schemas.
//! - `CustomFieldManager`: Registry for resolving and injecting custom fields into DocType definitions.
//! - `CustomDocType`: User-defined runtime DocType definitions.

use crate::schema::{DocFieldSchema, DocTypeSchema, FieldType, SchemaError};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime Custom Field definition attached to an existing DocType.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CustomField {
    /// Name of the target DocType being extended (e.g. "Sales Invoice").
    pub dt: CompactString,
    /// Unique field identifier on the target DocType.
    pub fieldname: CompactString,
    /// Human readable UI label.
    pub label: CompactString,
    /// Field type.
    pub fieldtype: FieldType,
    /// Optional field name after which this custom field should be rendered in UI forms.
    pub insert_after: Option<CompactString>,
    /// Mandatory requirement flag.
    #[serde(default)]
    pub reqd: bool,
    /// Read-only status.
    #[serde(default)]
    pub read_only: bool,
    /// Hidden flag.
    #[serde(default)]
    pub hidden: bool,
    /// Show in list view.
    #[serde(default)]
    pub in_list_view: bool,
    /// Optional selection options or Link target.
    #[serde(default)]
    pub options: Option<String>,
    /// Fallback default value.
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
}

impl CustomField {
    /// Converts this `CustomField` into a standard `DocFieldSchema`.
    #[must_use]
    pub fn to_doc_field_schema(&self) -> DocFieldSchema {
        DocFieldSchema {
            fieldname: self.fieldname.to_string(),
            fieldtype: self.fieldtype.clone(),
            label: self.label.to_string(),
            reqd: self.reqd,
            unique: false,
            read_only: self.read_only,
            hidden: self.hidden,
            in_list_view: self.in_list_view,
            mask: false,
            options: self.options.clone(),
            default_value: self.default_value.clone(),
            permlevel: 0,
            depends_on: None,
            mandatory_depends_on: None,
            read_only_depends_on: None,
            fetch_from: None,
            precision: None,
        }
    }
}

/// Registry and manager for dynamic custom fields across all DocTypes.
#[derive(Debug, Default, Clone)]
pub struct CustomFieldManager {
    /// Map of DocType name to registered custom fields.
    custom_fields: HashMap<CompactString, Vec<CustomField>>,
}

impl CustomFieldManager {
    /// Creates a new empty `CustomFieldManager`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a custom field for a target DocType.
    pub fn add_custom_field(&mut self, field: CustomField) -> Result<(), SchemaError> {
        let dt_key = field.dt.clone();
        let list = self.custom_fields.entry(dt_key).or_default();
        if list.iter().any(|f| f.fieldname == field.fieldname) {
            return Err(SchemaError::DuplicateFieldName(
                field.fieldname.to_string(),
                field.dt.to_string(),
            ));
        }
        list.push(field);
        Ok(())
    }

    /// Retrieves all custom fields registered for a given DocType.
    #[must_use]
    pub fn get_custom_fields(&self, doctype: &str) -> &[CustomField] {
        self.custom_fields
            .get(doctype)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Augments a base `DocTypeSchema` by inserting all registered custom fields.
    pub fn augment_doctype(&self, schema: &mut DocTypeSchema) {
        if let Some(custom_fields) = self.custom_fields.get(schema.name.as_str()) {
            for custom in custom_fields {
                let doc_field = custom.to_doc_field_schema();
                if let Some(ref after) = custom.insert_after
                    && let Some(idx) = schema
                        .fields
                        .iter()
                        .position(|f| f.fieldname == after.as_str())
                {
                    schema.fields.insert(idx + 1, doc_field);
                    continue;
                }
                schema.fields.push(doc_field);
            }
        }
    }

    /// Generates SurrealQL DDL statements to alter the database table for newly added custom fields.
    #[must_use]
    pub fn compile_custom_fields_ddl(&self) -> Vec<String> {
        let mut ddl = Vec::new();
        for (doctype, fields) in &self.custom_fields {
            let table = format!("tab_{}", doctype.to_lowercase().replace(' ', "_"));
            for field in fields {
                if field.fieldtype.is_layout_field() {
                    continue;
                }
                let stype = field.fieldtype.surreal_type();
                let field_sql = format!(
                    "DEFINE FIELD IF NOT EXISTS {} ON TABLE {} TYPE {};",
                    field.fieldname, table, stype
                );
                ddl.push(field_sql);
            }
        }
        ddl
    }
}

/// Dynamic user-defined runtime DocType representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomDocType {
    pub name: CompactString,
    pub module: CompactString,
    pub is_submittable: bool,
    pub is_child_table: bool,
    pub fields: Vec<CustomField>,
}

impl CustomDocType {
    /// Converts a user-defined custom DocType into a standard `DocTypeSchema`.
    #[must_use]
    pub fn into_doctype_schema(self) -> DocTypeSchema {
        DocTypeSchema {
            name: self.name.to_string(),
            module: self.module.to_string(),
            is_single: false,
            is_submittable: self.is_submittable,
            is_child_table: self.is_child_table,
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
            fields: self
                .fields
                .into_iter()
                .map(|f| f.to_doc_field_schema())
                .collect(),
            permissions: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_field_injection_and_positioning() {
        let mut manager = CustomFieldManager::new();
        manager
            .add_custom_field(CustomField {
                dt: "Customer".into(),
                fieldname: "loyalty_points".into(),
                label: "Loyalty Points".into(),
                fieldtype: FieldType::Int,
                insert_after: Some("customer_name".into()),
                reqd: false,
                read_only: false,
                hidden: false,
                in_list_view: true,
                options: None,
                default_value: Some(serde_json::json!(0)),
            })
            .expect("Failed to add custom field");

        let mut schema = DocTypeSchema {
            name: "Customer".into(),
            module: "Selling".into(),
            is_single: false,
            is_submittable: false,
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
            fields: vec![
                DocFieldSchema {
                    fieldname: "customer_name".into(),
                    label: "Customer Name".into(),
                    fieldtype: FieldType::Data,
                    reqd: true,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
                DocFieldSchema {
                    fieldname: "territory".into(),
                    label: "Territory".into(),
                    fieldtype: FieldType::Data,
                    reqd: false,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: false,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
            ],
            permissions: vec![],
        };

        manager.augment_doctype(&mut schema);
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[1].fieldname, "loyalty_points");

        let ddl = manager.compile_custom_fields_ddl();
        assert_eq!(ddl.len(), 1);
        assert!(
            ddl[0].contains(
                "DEFINE FIELD IF NOT EXISTS loyalty_points ON TABLE tab_customer TYPE int;"
            )
        );
    }
}
