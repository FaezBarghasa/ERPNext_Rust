//! Pure Dynamic No-Code Meta-Model Engine & Registry (`frappe-meta::dynamic_registry`).
//!
//! Enables non-technical users to create, modify, rename, and delete custom DocTypes and fields
//! directly at runtime without touching Rust code or restarting the server.
//!
//! Features:
//! - Thread-safe `Arc<RwLock<...>>` in-memory dynamic schema cache.
//! - Automatic validation of fields, naming conventions, and reserved keywords.
//! - Runtime document payload validation against dynamic schema definitions.
//! - Dynamic SurrealQL DDL compilation for runtime database table & field creation.
//! - JSON snapshot export and import for seamless backup and migration.

use crate::custom_fields::{CustomDocType, CustomField, CustomFieldManager};
use crate::registry::standard_erpnext_schemas;
use crate::schema::{DocTypeSchema, FieldType, SchemaError};
use compact_str::CompactString;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

static GLOBAL_REGISTRY: OnceLock<DynamicSchemaRegistry> = OnceLock::new();

/// Returns a shared reference to the global `DynamicSchemaRegistry`, pre-populated with standard schemas.
#[must_use]
pub fn global_schema_registry() -> &'static DynamicSchemaRegistry {
    GLOBAL_REGISTRY.get_or_init(DynamicSchemaRegistry::new_with_standard)
}

/// Thread-safe dynamic schema registry supporting zero-recompile runtime schema mutations.
#[derive(Debug, Clone)]
pub struct DynamicSchemaRegistry {
    schemas: Arc<RwLock<HashMap<CompactString, Arc<DocTypeSchema>>>>,
    custom_field_mgr: Arc<RwLock<CustomFieldManager>>,
}

impl Default for DynamicSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicSchemaRegistry {
    /// Creates a new empty dynamic schema registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            custom_field_mgr: Arc::new(RwLock::new(CustomFieldManager::new())),
        }
    }

    /// Initializes a dynamic schema registry pre-populated with standard business DocTypes.
    #[must_use]
    pub fn new_with_standard() -> Self {
        let registry = Self::new();
        for schema in standard_erpnext_schemas() {
            let _ = registry.register_schema(schema);
        }
        registry
    }

    /// Registers or updates a full `DocTypeSchema`.
    pub fn register_schema(
        &self,
        mut schema: DocTypeSchema,
    ) -> Result<Arc<DocTypeSchema>, SchemaError> {
        schema.validate()?;
        let key = CompactString::new(&schema.name);

        // Apply custom fields if any exist for this DocType
        if let Ok(cfm) = self.custom_field_mgr.read() {
            cfm.augment_doctype(&mut schema);
        }

        let arc_schema = Arc::new(schema);
        if let Ok(mut lock) = self.schemas.write() {
            lock.insert(key, arc_schema.clone());
        }
        Ok(arc_schema)
    }

    /// Dynamically creates a new user-defined custom DocType at runtime.
    pub fn create_custom_doctype(
        &self,
        custom: CustomDocType,
    ) -> Result<Arc<DocTypeSchema>, SchemaError> {
        let schema = custom.into_doctype_schema();
        self.register_schema(schema)
    }

    /// Dynamically updates an existing custom DocType at runtime.
    pub fn update_custom_doctype(
        &self,
        name: &str,
        custom: CustomDocType,
    ) -> Result<Arc<DocTypeSchema>, SchemaError> {
        let key = CompactString::new(name);
        if let Ok(lock) = self.schemas.read()
            && !lock.contains_key(&key)
        {
            return Err(SchemaError::BaseClassNotFound(
                name.to_string(),
                "DynamicSchemaRegistry".to_string(),
            ));
        }
        let schema = custom.into_doctype_schema();
        self.register_schema(schema)
    }

    /// Dynamically deletes a custom DocType from the registry.
    pub fn delete_custom_doctype(&self, name: &str) -> Result<bool, SchemaError> {
        let key = CompactString::new(name);
        if let Ok(mut lock) = self.schemas.write() {
            Ok(lock.remove(&key).is_some())
        } else {
            Ok(false)
        }
    }

    /// Retrieves an atomic reference to a registered DocType schema by name.
    #[must_use]
    pub fn get_schema(&self, name: &str) -> Option<Arc<DocTypeSchema>> {
        let lock = self.schemas.read().ok()?;
        if let Some(schema) = lock.get(name) {
            return Some(schema.clone());
        }
        // Case-insensitive fallback
        lock.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }

    /// Lists all currently registered DocType schemas.
    #[must_use]
    pub fn list_doctypes(&self) -> Vec<Arc<DocTypeSchema>> {
        self.schemas
            .read()
            .map(|lock| lock.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Dynamically attaches a custom field to a target DocType at runtime.
    pub fn add_custom_field(&self, field: CustomField) -> Result<(), SchemaError> {
        let target_dt = field.dt.to_string();
        if let Ok(mut cfm) = self.custom_field_mgr.write() {
            cfm.add_custom_field(field)?;
        }
        // Re-augment and re-register the target schema if it exists
        if let Some(schema) = self.get_schema(&target_dt) {
            let mut updated = (*schema).clone();
            if let Ok(cfm) = self.custom_field_mgr.read() {
                cfm.augment_doctype(&mut updated);
            }
            let key = CompactString::new(&target_dt);
            if let Ok(mut lock) = self.schemas.write() {
                lock.insert(key, Arc::new(updated));
            }
        }
        Ok(())
    }

    /// Validates an incoming document payload against the DocType schema rules.
    pub fn validate_document_payload(
        &self,
        doctype: &str,
        payload: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        let schema = self.get_schema(doctype).ok_or_else(|| {
            SchemaError::BaseClassNotFound(doctype.to_string(), "Registry".to_string())
        })?;

        let obj = payload.as_object().ok_or_else(|| {
            SchemaError::CompilationFailed(format!("Payload for '{doctype}' must be a JSON object"))
        })?;

        for field in &schema.fields {
            if field.fieldtype.is_layout_field() {
                continue;
            }

            let val = obj.get(&field.fieldname);

            // Mandatory requirement check
            if field.reqd {
                match val {
                    None | Some(serde_json::Value::Null) => {
                        return Err(SchemaError::CompilationFailed(format!(
                            "Missing mandatory field '{}' on DocType '{}'",
                            field.fieldname, schema.name
                        )));
                    }
                    Some(serde_json::Value::String(s)) if s.trim().is_empty() => {
                        return Err(SchemaError::CompilationFailed(format!(
                            "Mandatory field '{}' on DocType '{}' cannot be empty",
                            field.fieldname, schema.name
                        )));
                    }
                    _ => {}
                }
            }

            // Type verification if field is present
            if let Some(val) = val
                && !val.is_null()
            {
                match &field.fieldtype {
                    FieldType::Int => {
                        if !val.is_i64()
                            && !val
                                .as_str()
                                .map(|s| s.parse::<i64>().is_ok())
                                .unwrap_or(false)
                        {
                            return Err(SchemaError::CompilationFailed(format!(
                                "Field '{}' on DocType '{}' expects an integer, got: {val}",
                                field.fieldname, schema.name
                            )));
                        }
                    }
                    FieldType::Float | FieldType::Currency | FieldType::Percent => {
                        if !val.is_f64()
                            && !val.is_i64()
                            && !val
                                .as_str()
                                .map(|s| s.parse::<f64>().is_ok())
                                .unwrap_or(false)
                        {
                            return Err(SchemaError::CompilationFailed(format!(
                                "Field '{}' on DocType '{}' expects a number, got: {val}",
                                field.fieldname, schema.name
                            )));
                        }
                    }
                    FieldType::Check => {
                        if !val.is_boolean() && val.as_i64().map(|n| n == 0 || n == 1) != Some(true)
                        {
                            return Err(SchemaError::CompilationFailed(format!(
                                "Field '{}' on DocType '{}' expects a boolean or 0/1, got: {val}",
                                field.fieldname, schema.name
                            )));
                        }
                    }
                    FieldType::Table { child_doctype: _ } => {
                        if !val.is_array() {
                            return Err(SchemaError::CompilationFailed(format!(
                                "Child table field '{}' on DocType '{}' expects a JSON array of rows",
                                field.fieldname, schema.name
                            )));
                        }
                    }
                    FieldType::Link { target_doctype: _ } if !val.is_string() => {
                        return Err(SchemaError::CompilationFailed(format!(
                            "Link field '{}' on DocType '{}' expects a string identifier",
                            field.fieldname, schema.name
                        )));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Compiles complete SurrealQL DDL for a registered DocType.
    pub fn compile_surrealql_ddl(&self, doctype: &str) -> Result<Vec<String>, SchemaError> {
        let schema = self.get_schema(doctype).ok_or_else(|| {
            SchemaError::BaseClassNotFound(doctype.to_string(), "Registry".to_string())
        })?;

        let table = format!("tab_{}", schema.name.to_lowercase().replace(' ', "_"));
        let mut ddl = vec![format!("DEFINE TABLE IF NOT EXISTS {table} SCHEMALESS;")];

        for field in &schema.fields {
            if field.fieldtype.is_layout_field() {
                continue;
            }
            let stype = field.fieldtype.surreal_type();
            ddl.push(format!(
                "DEFINE FIELD IF NOT EXISTS {} ON TABLE {} TYPE {};",
                field.fieldname, table, stype
            ));
        }

        Ok(ddl)
    }

    /// Exports all dynamic schemas as a portable JSON snapshot.
    #[must_use]
    pub fn export_snapshot(&self) -> serde_json::Value {
        let doctypes: Vec<DocTypeSchema> =
            self.list_doctypes().iter().map(|d| (**d).clone()).collect();
        serde_json::json!({
            "version": "1.0",
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "doctypes": doctypes,
        })
    }

    /// Imports and registers an entire schema snapshot at runtime.
    pub fn import_snapshot(&self, snapshot: &serde_json::Value) -> Result<usize, SchemaError> {
        let doctypes_arr = snapshot
            .get("doctypes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                SchemaError::CompilationFailed("Invalid snapshot: missing 'doctypes' array".into())
            })?;

        let mut count = 0;
        for item in doctypes_arr {
            let schema: DocTypeSchema = serde_json::from_value(item.clone()).map_err(|e| {
                SchemaError::CompilationFailed(format!("Failed to parse DocType schema: {e}"))
            })?;
            self.register_schema(schema)?;
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_custom_doctype_creation_and_retrieval() {
        let registry = DynamicSchemaRegistry::new();
        let custom = CustomDocType {
            name: "CourseLesson".into(),
            module: "LMS".into(),
            is_submittable: false,
            is_child_table: false,
            fields: vec![
                CustomField {
                    dt: "CourseLesson".into(),
                    fieldname: "title".into(),
                    label: "Lesson Title".into(),
                    fieldtype: FieldType::Data,
                    insert_after: None,
                    reqd: true,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    options: None,
                    default_value: None,
                },
                CustomField {
                    dt: "CourseLesson".into(),
                    fieldname: "duration_minutes".into(),
                    label: "Duration (min)".into(),
                    fieldtype: FieldType::Int,
                    insert_after: Some("title".into()),
                    reqd: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    options: None,
                    default_value: Some(serde_json::json!(15)),
                },
            ],
        };

        let schema = registry
            .create_custom_doctype(custom)
            .expect("Failed to create custom DocType");
        assert_eq!(schema.name, "CourseLesson");
        assert_eq!(schema.fields.len(), 2);

        // Retrieve schema
        let fetched = registry
            .get_schema("CourseLesson")
            .expect("DocType not found");
        assert_eq!(fetched.module, "LMS");

        // Validate payload happy path
        let valid_payload = serde_json::json!({
            "title": "Introduction to Rust Systems",
            "duration_minutes": 30
        });
        assert!(
            registry
                .validate_document_payload("CourseLesson", &valid_payload)
                .is_ok()
        );

        // Validate payload missing mandatory
        let invalid_payload = serde_json::json!({
            "duration_minutes": 30
        });
        assert!(
            registry
                .validate_document_payload("CourseLesson", &invalid_payload)
                .is_err()
        );
    }

    #[test]
    fn test_dynamic_surrealql_ddl_compilation() {
        let registry = DynamicSchemaRegistry::new();
        let custom = CustomDocType {
            name: "SaaSSubscription".into(),
            module: "Billing".into(),
            is_submittable: true,
            is_child_table: false,
            fields: vec![
                CustomField {
                    dt: "SaaSSubscription".into(),
                    fieldname: "customer_id".into(),
                    label: "Customer ID".into(),
                    fieldtype: FieldType::Link {
                        target_doctype: "Customer".into(),
                    },
                    insert_after: None,
                    reqd: true,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    options: Some("Customer".into()),
                    default_value: None,
                },
                CustomField {
                    dt: "SaaSSubscription".into(),
                    fieldname: "monthly_rate".into(),
                    label: "Monthly Rate".into(),
                    fieldtype: FieldType::Currency,
                    insert_after: None,
                    reqd: true,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    options: None,
                    default_value: None,
                },
            ],
        };

        registry.create_custom_doctype(custom).unwrap();
        let ddl = registry.compile_surrealql_ddl("SaaSSubscription").unwrap();
        assert!(
            ddl.iter()
                .any(|stmt| stmt.contains("DEFINE TABLE IF NOT EXISTS tab_saassubscription"))
        );
        assert!(
            ddl.iter()
                .any(|stmt| stmt.contains("DEFINE FIELD IF NOT EXISTS customer_id"))
        );
        assert!(
            ddl.iter()
                .any(|stmt| stmt.contains("DEFINE FIELD IF NOT EXISTS monthly_rate"))
        );
    }

    #[test]
    fn test_schema_snapshot_export_and_import() {
        let registry1 = DynamicSchemaRegistry::new();
        let custom = CustomDocType {
            name: "TestArticle".into(),
            module: "CMS".into(),
            is_submittable: false,
            is_child_table: false,
            fields: vec![CustomField {
                dt: "TestArticle".into(),
                fieldname: "headline".into(),
                label: "Headline".into(),
                fieldtype: FieldType::Data,
                insert_after: None,
                reqd: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                options: None,
                default_value: None,
            }],
        };
        registry1.create_custom_doctype(custom).unwrap();
        let snapshot = registry1.export_snapshot();

        let registry2 = DynamicSchemaRegistry::new();
        let imported_count = registry2.import_snapshot(&snapshot).unwrap();
        assert_eq!(imported_count, 1);
        assert!(registry2.get_schema("TestArticle").is_some());
    }
}
