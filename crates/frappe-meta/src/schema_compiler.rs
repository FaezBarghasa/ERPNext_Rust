use crate::schema::{DocFieldSchema, DocTypeSchema, FieldType, SchemaError};

/// Compiles a conditional JS / Frappe expression into a normalized SurrealQL boolean expression if possible.
#[must_use]
pub fn compile_expression_to_surrealql(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return "true".into();
    }
    // Handle "eval:doc.status == 'Draft'" or "doc.is_return == 1"
    let clean = trimmed.strip_prefix("eval:").unwrap_or(trimmed).trim();
    clean
        .replace("doc.", "")
        .replace("===", "=")
        .replace("==", "=")
        .replace("&&", "AND")
        .replace("||", "OR")
}

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

    // Nested Set Tree support (lft, rgt, is_group)
    if schema.is_tree {
        statements.push(format!(
            "DEFINE FIELD lft ON TABLE {table} TYPE int DEFAULT 0;"
        ));
        statements.push(format!(
            "DEFINE FIELD rgt ON TABLE {table} TYPE int DEFAULT 0;"
        ));
        statements.push(format!(
            "DEFINE FIELD is_group ON TABLE {table} TYPE bool DEFAULT false;"
        ));
        statements.push(format!(
            "DEFINE FIELD parent_{table} ON TABLE {table} TYPE option<record<{table}>>;"
        ));
    }

    // Custom fields from schema
    for field in &schema.fields {
        // Skip layout-only primitives (SectionBreak, ColumnBreak, TabBreak, Button) from DB column definitions
        if field.fieldtype.is_layout_field() {
            continue;
        }

        let field_ddl = compile_field_ddl(&table, field)?;
        statements.push(field_ddl);

        if field.unique {
            statements.push(format!(
                "DEFINE INDEX idx_{table}_{field_name} ON TABLE {table} FIELDS {field_name} UNIQUE;",
                field_name = field.fieldname
            ));
        }
    }

    // Search fields indexes
    if let Some(search_fields) = &schema.search_fields {
        let fields: Vec<&str> = search_fields
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for sf in fields {
            // Verify if field exists in schema
            if schema.fields.iter().any(|f| f.fieldname == sf) {
                statements.push(format!(
                    "DEFINE INDEX idx_{table}_search_{sf} ON TABLE {table} FIELDS {sf};"
                ));
            }
        }
    }

    // Secondary composite indexes for hot ledger tables (general ledger and stock ledger)
    if table == "gl_entry" || table == "gl_entries" {
        statements.push(format!(
            "DEFINE INDEX idx_{table}_account_posting_date ON TABLE {table} FIELDS account, posting_date;"
        ));
        statements.push(format!(
            "DEFINE INDEX idx_{table}_voucher ON TABLE {table} FIELDS voucher_type, voucher_no;"
        ));
    } else if table == "stock_ledger_entry" || table == "stock_ledger_entries" {
        statements.push(format!(
            "DEFINE INDEX idx_{table}_item_warehouse ON TABLE {table} FIELDS item_code, warehouse;"
        ));
        statements.push(format!(
            "DEFINE INDEX idx_{table}_posting_date ON TABLE {table} FIELDS posting_date;"
        ));
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
        if !formatted_opts.is_empty() {
            assertions.push(format!("$value IN [{formatted_opts}]"));
        }
    }

    // Link foreign key constraint assertion
    if let FieldType::Link { .. } = &field.fieldtype {
        assertions
            .push("($value == NONE OR $value == NULL OR type::is_record($value))".to_string());
    }

    // Numerical range / non-negativity invariants
    match &field.fieldtype {
        FieldType::Percent => {
            assertions.push("($value == NONE OR ($value >= 0 AND $value <= 100))".to_string());
        }
        FieldType::Rating => {
            assertions.push("($value == NONE OR ($value >= 0 AND $value <= 5))".to_string());
        }
        FieldType::Currency | FieldType::Float | FieldType::Int => {
            let fname = field.fieldname.to_ascii_lowercase();
            if fname == "qty"
                || fname == "rate"
                || fname == "standard_rate"
                || fname == "price_list_rate"
                || fname.ends_with("_qty")
                || fname.ends_with("_rate")
                || fname.starts_with("qty_")
            {
                assertions.push("($value == NONE OR $value >= 0)".to_string());
            }
        }
        FieldType::SmallText => {
            assertions.push("($value == NONE OR string::len($value) <= 250)".to_string());
        }
        FieldType::DataWithPreset { preset } => match preset {
            crate::schema::DataPreset::Email => {
                assertions.push("($value == NONE OR string::is::email($value))".to_string());
            }
            crate::schema::DataPreset::Url => {
                assertions.push("($value == NONE OR string::is::url($value))".to_string());
            }
            _ => {}
        },
        _ => {}
    }

    if !assertions.is_empty() {
        parts.push(format!("ASSERT {}", assertions.join(" AND ")));
    }

    parts.push(";".to_string());
    Ok(parts.join(" "))
}
