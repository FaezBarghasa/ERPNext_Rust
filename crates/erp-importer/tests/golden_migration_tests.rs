//! Upstream MariaDB to SurrealDB Migration & Parity Golden Tests (`erp-importer`).
//!
//! Validates Phase 8 requirements:
//! 1. MariaDB `tab*` tables parsing and normalization into canonical SurrealDB documents.
//! 2. Preservation of metadata: `name`, `docstatus` (0=Draft, 1=Submitted, 2=Cancelled), `creation`, `modified`, `owner`.
//! 3. Automatic child table rows grouping and embedding (`parent`, `parentfield`, `parenttype`, `idx`).
//! 4. Automated zero-drift financial reconciliation (GL debits == credits, source == imported).
//! 5. Automated zero-drift inventory reconciliation (Stock Ledger actual_qty and stock_value_difference).
//! 6. Dry-run execution mode verification.

use erp_importer::{MariaDbImportConfig, MariaDbMigrationEngine, MariaDbTableDump};
use rust_decimal_macros::dec;
use serde_json::json;

#[test]
fn test_golden_mariadb_migration_and_reconciliation_flow() {
    // 1. Mock `tabItem` (Master records)
    let item_table = MariaDbTableDump {
        table_name: "tabItem".into(),
        columns: vec![
            "name".into(),
            "item_name".into(),
            "item_group".into(),
            "stock_uom".into(),
            "valuation_rate".into(),
            "creation".into(),
            "modified".into(),
            "docstatus".into(),
        ],
        rows: vec![
            vec![
                json!("ITEM-ARM-M4"),
                json!("Cortex-M4 Embedded Controller"),
                json!("Electronics"),
                json!("Nos"),
                json!(150.0),
                json!("2026-01-01 10:00:00"),
                json!("2026-01-01 10:00:00"),
                json!(0),
            ],
            vec![
                json!("ITEM-CAN-SENS"),
                json!("CANbus Sensor Node"),
                json!("Electronics"),
                json!("Nos"),
                json!(45.0),
                json!("2026-01-02 11:00:00"),
                json!("2026-01-02 11:00:00"),
                json!(0),
            ],
        ],
    };

    // 2. Mock `tabSales Invoice` (Parent records)
    let si_parent_table = MariaDbTableDump {
        table_name: "tabSales Invoice".into(),
        columns: vec![
            "name".into(),
            "customer".into(),
            "posting_date".into(),
            "grand_total".into(),
            "docstatus".into(),
            "creation".into(),
            "modified".into(),
        ],
        rows: vec![vec![
            json!("ACC-SINV-2026-00001"),
            json!("Globex Robotics Corp"),
            json!("2026-10-16"),
            json!(1950.0),
            json!(1), // Submitted
            json!("2026-10-16 12:00:00"),
            json!("2026-10-16 12:00:00"),
        ]],
    };

    // 3. Mock `tabSales Invoice Item` (Child records with parent reference)
    let si_child_table = MariaDbTableDump {
        table_name: "tabSales Invoice Item".into(),
        columns: vec![
            "name".into(),
            "parent".into(),
            "parenttype".into(),
            "parentfield".into(),
            "idx".into(),
            "item_code".into(),
            "qty".into(),
            "rate".into(),
            "amount".into(),
        ],
        rows: vec![
            vec![
                json!("ROW-001"),
                json!("ACC-SINV-2026-00001"),
                json!("Sales Invoice"),
                json!("items"),
                json!(1),
                json!("ITEM-ARM-M4"),
                json!(10.0),
                json!(150.0),
                json!(1500.0),
            ],
            vec![
                json!("ROW-002"),
                json!("ACC-SINV-2026-00001"),
                json!("Sales Invoice"),
                json!("items"),
                json!(2),
                json!("ITEM-CAN-SENS"),
                json!(10.0),
                json!(45.0),
                json!(450.0),
            ],
        ],
    };

    // 4. Mock `tabGL Entry` (Financial Ledger)
    // Debit: Debtors $1950, Credit: Sales $1950 -> balanced!
    let gl_table = MariaDbTableDump {
        table_name: "tabGL Entry".into(),
        columns: vec![
            "name".into(),
            "voucher_type".into(),
            "voucher_no".into(),
            "account".into(),
            "debit".into(),
            "credit".into(),
            "posting_date".into(),
        ],
        rows: vec![
            vec![
                json!("GL-001"),
                json!("Sales Invoice"),
                json!("ACC-SINV-2026-00001"),
                json!("1310 - Accounts Receivable"),
                json!(1950.0),
                json!(0.0),
                json!("2026-10-16"),
            ],
            vec![
                json!("GL-002"),
                json!("Sales Invoice"),
                json!("ACC-SINV-2026-00001"),
                json!("4110 - Sales Revenue"),
                json!(0.0),
                json!(1950.0),
                json!("2026-10-16"),
            ],
        ],
    };

    // 5. Mock `tabStock Ledger Entry` (Inventory Ledger)
    let sle_table = MariaDbTableDump {
        table_name: "tabStock Ledger Entry".into(),
        columns: vec![
            "name".into(),
            "item_code".into(),
            "warehouse".into(),
            "actual_qty".into(),
            "stock_value_difference".into(),
            "voucher_type".into(),
            "voucher_no".into(),
        ],
        rows: vec![
            vec![
                json!("SLE-001"),
                json!("ITEM-ARM-M4"),
                json!("Stores - A"),
                json!(-10.0),
                json!(-1500.0),
                json!("Sales Invoice"),
                json!("ACC-SINV-2026-00001"),
            ],
            vec![
                json!("SLE-002"),
                json!("ITEM-CAN-SENS"),
                json!("Stores - A"),
                json!(-10.0),
                json!(-450.0),
                json!("Sales Invoice"),
                json!("ACC-SINV-2026-00001"),
            ],
        ],
    };

    let tables = vec![
        item_table,
        si_parent_table,
        si_child_table,
        gl_table,
        sle_table,
    ];

    let config = MariaDbImportConfig {
        dry_run: false,
        batch_size: 1000,
        reconcile_gl: true,
        reconcile_stock: true,
    };

    // Execute Migration
    let (documents, report) = MariaDbMigrationEngine::process_migration(&tables, &config);

    // 1. Verify Parent Documents & Child Rows Embedding
    let invoices = documents.get("Sales Invoice").expect("Sales Invoice docs");
    assert_eq!(invoices.len(), 1);

    let invoice = &invoices[0];
    assert_eq!(
        invoice.get("name").and_then(|v| v.as_str()),
        Some("ACC-SINV-2026-00001")
    );
    assert_eq!(invoice.get("docstatus").and_then(|v| v.as_i64()), Some(1));

    // Verify child table `items` is embedded as a JSON array with 2 rows in correct order
    let items = invoice
        .get("items")
        .and_then(|v| v.as_array())
        .expect("items array");
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].get("item_code").and_then(|v| v.as_str()),
        Some("ITEM-ARM-M4")
    );
    assert_eq!(
        items[1].get("item_code").and_then(|v| v.as_str()),
        Some("ITEM-CAN-SENS")
    );

    // 2. Verify General Ledger Reconciliation
    assert_eq!(report.gl_debits_source, dec!(1950.0));
    assert_eq!(report.gl_debits_imported, dec!(1950.0));
    assert_eq!(report.gl_credits_source, dec!(1950.0));
    assert_eq!(report.gl_credits_imported, dec!(1950.0));

    // 3. Verify Stock Ledger Reconciliation
    assert_eq!(report.stock_qty_source, dec!(-20.0));
    assert_eq!(report.stock_qty_imported, dec!(-20.0));
    assert_eq!(report.stock_val_source, dec!(-1950.0));
    assert_eq!(report.stock_val_imported, dec!(-1950.0));

    // 4. Verify Complete Zero-Drift Parity
    assert!(report.is_reconciled);
    assert!(report.discrepancies.is_empty());
}

#[test]
fn test_golden_mariadb_dry_run_mode() {
    let item_table = MariaDbTableDump {
        table_name: "tabItem".into(),
        columns: vec!["name".into(), "docstatus".into()],
        rows: vec![vec![json!("ITEM-001"), json!(0)]],
    };

    let config = MariaDbImportConfig {
        dry_run: true,
        batch_size: 500,
        reconcile_gl: false,
        reconcile_stock: false,
    };

    let (_docs, report) = MariaDbMigrationEngine::process_migration(&[item_table], &config);
    assert!(report.dry_run);
    assert!(report.is_reconciled);
}
