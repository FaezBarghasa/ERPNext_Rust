use crate::schema::{DocFieldSchema, DocTypeSchema, SchemaError};
use std::collections::HashMap;

/// Representation of an existing field in SurrealDB live dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingFieldDef {
    /// Field identifier.
    pub name: String,
    /// SurrealQL data type.
    pub field_type: String,
    /// Is this field constrained by assertions?
    pub has_assertion: bool,
}

/// Representation of an existing table definition in SurrealDB.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExistingTableDef {
    /// Table name.
    pub name: String,
    /// Map of field names to their live definitions.
    pub fields: HashMap<String, ExistingFieldDef>,
}

/// Snapshot of the live database dictionary.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExistingDatabaseSchema {
    /// Map of table names to their table definitions.
    pub tables: HashMap<String, ExistingTableDef>,
}

/// Calculated structural difference between target schema and live database.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MigrationDelta {
    /// Table name being modified.
    pub table_name: String,
    /// Is the table newly created?
    pub is_new_table: bool,
    /// Fields to be added.
    pub added_fields: Vec<DocFieldSchema>,
    /// Fields to be removed.
    pub removed_fields: Vec<String>,
    /// Fields with changed types or constraints.
    pub modified_fields: Vec<DocFieldSchema>,
}

/// Calculates the structural diff between a target DocType and live database schema.
pub fn diff_schema(
    target: &DocTypeSchema,
    existing_db: &ExistingDatabaseSchema,
) -> Result<MigrationDelta, SchemaError> {
    target.validate()?;
    let table_name = target.table_name();

    let mut delta = MigrationDelta {
        table_name: table_name.clone(),
        ..Default::default()
    };

    if let Some(existing_table) = existing_db.tables.get(&table_name) {
        let mut target_fields_map = HashMap::new();
        for field in &target.fields {
            target_fields_map.insert(field.fieldname.clone(), field);
            if let Some(existing_field) = existing_table.fields.get(&field.fieldname) {
                let target_type = field.fieldtype.surreal_type();
                if existing_field.field_type != target_type {
                    delta.modified_fields.push(field.clone());
                }
            } else {
                delta.added_fields.push(field.clone());
            }
        }

        for existing_field_name in existing_table.fields.keys() {
            // Ignore standard Frappe metadata fields during drop diff
            if matches!(
                existing_field_name.as_str(),
                "docstatus" | "idx" | "owner" | "creation" | "modified" | "modified_by" | "id"
            ) {
                continue;
            }
            if !target_fields_map.contains_key(existing_field_name) {
                delta.removed_fields.push(existing_field_name.clone());
            }
        }
    } else {
        delta.is_new_table = true;
        delta.added_fields = target.fields.clone();
    }

    Ok(delta)
}

/// Generates transactional forward migration DDL and corresponding reverse rollback DDL.
pub fn generate_migration_ddl(
    target: &DocTypeSchema,
    delta: &MigrationDelta,
) -> (Vec<String>, Vec<String>) {
    let table = &delta.table_name;
    let mut forward = vec!["BEGIN TRANSACTION;".to_string()];
    let mut rollback = vec!["BEGIN TRANSACTION;".to_string()];

    if delta.is_new_table {
        forward.push(format!("DEFINE TABLE {table} SCHEMAFULL;"));
        rollback.push(format!("REMOVE TABLE {table};"));

        for field in &delta.added_fields {
            let type_str = field.fieldtype.surreal_type();
            forward.push(format!(
                "DEFINE FIELD {name} ON TABLE {table} TYPE {type_str};",
                name = field.fieldname
            ));
        }
    } else {
        for field in &delta.added_fields {
            let type_str = field.fieldtype.surreal_type();
            forward.push(format!(
                "DEFINE FIELD {name} ON TABLE {table} TYPE {type_str};",
                name = field.fieldname
            ));
            rollback.push(format!(
                "REMOVE FIELD {name} ON TABLE {table};",
                name = field.fieldname
            ));
        }

        for field_name in &delta.removed_fields {
            forward.push(format!("REMOVE FIELD {field_name} ON TABLE {table};"));
            rollback.push(format!(
                "DEFINE FIELD {field_name} ON TABLE {table} TYPE string;",
            ));
        }

        for field in &delta.modified_fields {
            let type_str = field.fieldtype.surreal_type();
            forward.push(format!(
                "DEFINE FIELD {name} ON TABLE {table} TYPE {type_str} OVERWRITE;",
                name = field.fieldname
            ));
            rollback.push(format!(
                "DEFINE FIELD {name} ON TABLE {table} TYPE string OVERWRITE;",
                name = field.fieldname
            ));
        }
    }

    forward.push(format!(
        "UPDATE __schema_migrations SET last_applied = time::now() WHERE doctype = '{name}';",
        name = target.name
    ));
    forward.push("COMMIT TRANSACTION;".to_string());
    rollback.push("COMMIT TRANSACTION;".to_string());

    (forward, rollback)
}
