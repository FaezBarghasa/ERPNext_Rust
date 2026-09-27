use crate::schema::{DocFieldSchema, DocTypeSchema, FieldType, SchemaError};

/// Compiles a strongly typed `DocTypeSchema` into a sequence of enforceable SurrealQL DDL statements.
pub fn compile_to_surrealql(schema: &DocTypeSchema) -> Result<Vec<String>, SchemaError> {
    schema.validate()?;

    let mut statements = Vec::new();
    let table = schema.table_name();

    // Define table
    statements.push(format!("DEFINE TABLE {table} SCHEMAFULL;"));

    // Standard Frappe metadata fields
    statements.push(format!(
        "DEFINE FIELD docstatus ON TABLE {table} TYPE int DEFAULT 0 ASSERT $value IN [0, 1, 2];"
    ));
    statements.push(format!(
        "DEFINE FIELD idx ON TABLE {table} TYPE int DEFAULT 0;"
    ));
    statements.push(format!(
        "DEFINE FIELD owner ON TABLE {table} TYPE string DEFAULT $auth.id;"
    ));
    statements.push(format!(
        "DEFINE FIELD creation ON TABLE {table} TYPE datetime DEFAULT time::now();"
    ));
    statements.push(format!(
        "DEFINE FIELD modified ON TABLE {table} TYPE datetime DEFAULT time::now();"
    ));
    statements.push(format!(
        "DEFINE FIELD modified_by ON TABLE {table} TYPE string DEFAULT $auth.id;"
    ));

    // Custom fields from schema
    for field in &schema.fields {
        let field_ddl = compile_field_ddl(&table, field)?;
        statements.push(field_ddl);

        if field.unique {
            statements.push(format!(
                "DEFINE INDEX idx_{table}_{field_name} ON TABLE {table} FIELDS {field_name} UNIQUE;",
                field_name = field.fieldname
            ));
        }
    }

    Ok(statements)
}

fn compile_field_ddl(table: &str, field: &DocFieldSchema) -> Result<String, SchemaError> {
    let type_str = field.fieldtype.surreal_type();
    let mut parts = vec![format!(
        "DEFINE FIELD {name} ON TABLE {table} TYPE {type_str}",
        name = field.fieldname
    )];

    if let Some(default) = &field.default_value {
        let default_str = match default {
            serde_json::Value::String(s) => format!("\"{s}\""),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            other => other.to_string(),
        };
        parts.push(format!("DEFAULT {default_str}"));
    } else if let FieldType::Check = field.fieldtype {
        parts.push("DEFAULT false".to_string());
    }

    let mut assertions = Vec::new();

    if field.reqd {
        assertions.push("$value != NONE AND $value != NULL".to_string());
    }

    if let FieldType::Select { options } = &field.fieldtype {
        let formatted_opts = options
            .iter()
            .map(|o| format!("\"{o}\""))
            .collect::<Vec<_>>()
            .join(", ");
        assertions.push(format!("$value IN [{formatted_opts}]"));
    }

    if !assertions.is_empty() {
        parts.push(format!("ASSERT {}", assertions.join(" AND ")));
    }

    parts.push(";".to_string());
    Ok(parts.join(" "))
}
