use frappe_meta::{
    DocFieldSchema, DocPermSchema, DocTypeSchema, ExistingDatabaseSchema, ExistingFieldDef,
    ExistingTableDef, FieldType, SchemaError, compile_to_surrealql, diff_schema,
    generate_migration_ddl,
};

fn create_customer_schema() -> DocTypeSchema {
    DocTypeSchema {
        name: "Customer".to_string(),
        module: "Selling".to_string(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: true,
        quick_entry: false,
        allow_rename: false,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: Some("CUST-.#####".to_string()),
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        fields: vec![
            DocFieldSchema {
                fieldname: "customer_name".to_string(),
                fieldtype: FieldType::Data,
                label: "Customer Name".to_string(),
                reqd: true,
                unique: true,
                read_only: false,
                hidden: false,
                in_list_view: true,
                mask: false,
                options: None,
                default_value: None,
                permlevel: 0,
            },
            DocFieldSchema {
                fieldname: "credit_limit".to_string(),
                fieldtype: FieldType::Currency,
                label: "Credit Limit".to_string(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                options: None,
                default_value: Some(serde_json::json!(0.0)),
                permlevel: 0,
            },
            DocFieldSchema {
                fieldname: "default_currency".to_string(),
                fieldtype: FieldType::Link {
                    target_doctype: "Currency".to_string(),
                },
                label: "Currency".to_string(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                options: None,
                default_value: None,
                permlevel: 0,
            },
            DocFieldSchema {
                fieldname: "disabled".to_string(),
                fieldtype: FieldType::Check,
                label: "Disabled".to_string(),
                reqd: false,
                unique: false,
                read_only: false,
                hidden: false,
                in_list_view: false,
                mask: false,
                options: None,
                default_value: Some(serde_json::json!(false)),
                permlevel: 0,
            },
        ],
        permissions: vec![DocPermSchema {
            role: "Sales User".to_string(),
            read: true,
            write: true,
            create: true,
            delete: false,
            submit: false,
            cancel: false,
            amend: false,
            report: true,
            export: false,
            import: false,
            permlevel: 0,
            accounting_period_exempt: false,
        }],
    }
}

#[test]
fn test_schema_deserialization_and_compilation() {
    let customer = create_customer_schema();
    assert!(customer.validate().is_ok());

    let ddl = compile_to_surrealql(&customer).expect("DDL compilation failed");
    assert!(
        ddl.iter()
            .any(|s| s.contains("DEFINE TABLE customer SCHEMAFULL;"))
    );
    assert!(
        ddl.iter()
            .any(|s| s.contains("DEFINE FIELD customer_name ON TABLE customer TYPE string"))
    );
    assert!(
        ddl.iter()
            .any(|s| s.contains("ASSERT $value != NONE AND $value != NULL"))
    );
    assert!(ddl.iter().any(|s| s.contains(
        "DEFINE INDEX idx_customer_customer_name ON TABLE customer FIELDS customer_name UNIQUE;"
    )));
}

#[test]
fn test_reserved_keyword_rejection() {
    let invalid = DocTypeSchema {
        name: "TestDoc".to_string(),
        module: "Core".to_string(),
        is_single: false,
        is_submittable: false,
        is_child_table: false,
        is_tree: false,
        track_changes: false,
        quick_entry: false,
        allow_rename: false,
        allow_import: true,
        allow_auto_repeat: false,
        naming_rule: None,
        naming_rule_spec: None,
        virtual_child_tables: false,
        lazy_materialization: false,
        extends_class: None,
        fields: vec![DocFieldSchema {
            fieldname: "table".to_string(),
            fieldtype: FieldType::Data,
            label: "Table Field".to_string(),
            reqd: false,
            unique: false,
            read_only: false,
            hidden: false,
            in_list_view: false,
            mask: false,
            options: None,
            default_value: None,
            permlevel: 0,
        }],
        permissions: vec![],
    };

    assert_eq!(
        invalid.validate(),
        Err(SchemaError::ReservedKeyword("table".to_string()))
    );
}

#[test]
fn test_schema_migration_diff_and_rollback() {
    let target = create_customer_schema();
    let mut existing_db = ExistingDatabaseSchema::default();

    let mut customer_fields = std::collections::HashMap::new();
    customer_fields.insert(
        "customer_name".to_string(),
        ExistingFieldDef {
            name: "customer_name".to_string(),
            field_type: "string".to_string(),
            has_assertion: true,
        },
    );

    existing_db.tables.insert(
        "customer".to_string(),
        ExistingTableDef {
            name: "customer".to_string(),
            fields: customer_fields,
        },
    );

    let delta = diff_schema(&target, &existing_db).expect("Diff failed");
    assert!(!delta.is_new_table);
    assert_eq!(delta.added_fields.len(), 3); // credit_limit, default_currency, disabled

    let (forward, rollback) = generate_migration_ddl(&target, &delta);
    assert!(forward.first().unwrap().contains("BEGIN TRANSACTION"));
    assert!(forward.last().unwrap().contains("COMMIT TRANSACTION"));
    assert!(
        forward
            .iter()
            .any(|s| s.contains("DEFINE FIELD credit_limit ON TABLE customer TYPE decimal;"))
    );
    assert!(
        rollback
            .iter()
            .any(|s| s.contains("REMOVE FIELD credit_limit ON TABLE customer;"))
    );
}
