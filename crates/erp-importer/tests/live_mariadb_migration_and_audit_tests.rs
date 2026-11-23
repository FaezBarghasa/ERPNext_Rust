//! Integration tests verifying Live MariaDB Direct Streaming Migration & Reconciliation Auditor.
//!
//! Validates:
//! 1. Keyset streaming batching and MySQL data normalization (datetime(6), decimal(21,9), tinyint(1), longtext).
//! 2. Relational child table nesting (`tabSales Invoice Item` -> embedded `items: [...]`).
//! 3. Side-by-side GL debits/credits balance verification ($0.00 exact).
//! 4. Stock ledger quantities and valuation reconciliation.
//! 5. Trial balance account-by-account zero delta verification.
//! 6. Deterministic SHA256 document checksum and open count parity.

use compact_str::CompactString;
use erp_importer::live_streamer::{
    KeysetBatch, LiveMariaDbStreamer, MariaDbColumnMeta, MariaDbColumnType, MariaDbDirectSource,
    MariaDbTableSchema,
};
use erp_importer::reconciliation_auditor::ReconciliationAuditor;
use rust_decimal_macros::dec;
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// Mock Direct MariaDB TCP/Wire source simulator.
struct MockMariaDbDriver {
    pub tables: HashMap<String, (MariaDbTableSchema, Vec<Map<String, Value>>)>,
}

impl MariaDbDirectSource for MockMariaDbDriver {
    async fn discover_schemas(&self) -> Result<Vec<MariaDbTableSchema>, String> {
        Ok(self.tables.values().map(|(s, _)| s.clone()).collect())
    }

    async fn fetch_keyset_batch(
        &self,
        table: &str,
        pk: &str,
        cursor: Option<&str>,
        batch_size: usize,
    ) -> Result<KeysetBatch<Map<String, Value>>, String> {
        let Some((_, rows)) = self.tables.get(table) else {
            return Err(format!("Table {table} not found"));
        };

        let start_idx = if let Some(cur) = cursor {
            rows.iter()
                .position(|r| r.get(pk).and_then(|v| v.as_str()) == Some(cur))
                .map(|p| p + 1)
                .unwrap_or(0)
        } else {
            0
        };

        let end_idx = (start_idx + batch_size).min(rows.len());
        let slice = rows[start_idx..end_idx].to_vec();
        let last_cursor = slice.last().and_then(|r| r.get(pk).and_then(|v| v.as_str()).map(CompactString::from));
        let has_more = end_idx < rows.len();

        Ok(KeysetBatch {
            records: slice,
            last_cursor,
            has_more,
            total_processed: end_idx - start_idx,
        })
    }
}

#[tokio::test]
async fn test_live_mariadb_streaming_and_reconciliation_parity() {
    // 1. Build schema for tabSales Invoice (Parent)
    let parent_schema = MariaDbTableSchema {
        table_name: "tabSales Invoice".into(),
        doctype_name: "Sales Invoice".into(),
        is_child_table: false,
        primary_key_column: "name".into(),
        columns: vec![
            MariaDbColumnMeta {
                column_name: "name".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: true,
            },
            MariaDbColumnMeta {
                column_name: "customer".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "posting_date".into(),
                data_type: MariaDbColumnType::DateOnly,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "grand_total".into(),
                data_type: MariaDbColumnType::DecimalVal,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "creation".into(),
                data_type: MariaDbColumnType::DateTimeUtc,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "is_return".into(),
                data_type: MariaDbColumnType::TinyIntBool,
                is_nullable: false,
                is_primary_key: false,
            },
        ],
    };

    // 2. Build schema for tabSales Invoice Item (Child)
    let child_schema = MariaDbTableSchema {
        table_name: "tabSales Invoice Item".into(),
        doctype_name: "Sales Invoice Item".into(),
        is_child_table: true,
        primary_key_column: "name".into(),
        columns: vec![
            MariaDbColumnMeta {
                column_name: "name".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: true,
            },
            MariaDbColumnMeta {
                column_name: "parent".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "parenttype".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "parentfield".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "idx".into(),
                data_type: MariaDbColumnType::Integer,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "item_code".into(),
                data_type: MariaDbColumnType::Varchar,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "qty".into(),
                data_type: MariaDbColumnType::DecimalVal,
                is_nullable: false,
                is_primary_key: false,
            },
            MariaDbColumnMeta {
                column_name: "rate".into(),
                data_type: MariaDbColumnType::DecimalVal,
                is_nullable: false,
                is_primary_key: false,
            },
        ],
    };

    // 3. Raw MySQL rows with unparsed MySQL data representations
    let mut raw_parent_row1 = Map::new();
    raw_parent_row1.insert("name".into(), json!("ACC-SINV-2026-00001"));
    raw_parent_row1.insert("customer".into(), json!("Acme Robotics"));
    raw_parent_row1.insert("posting_date".into(), json!("2026-10-16"));
    raw_parent_row1.insert("grand_total".into(), json!("1500.500000000"));
    raw_parent_row1.insert("creation".into(), json!("2026-10-16 12:30:00.123456"));
    raw_parent_row1.insert("is_return".into(), json!(0));

    let mut raw_child_row1 = Map::new();
    raw_child_row1.insert("name".into(), json!("ROW-1"));
    raw_child_row1.insert("parent".into(), json!("ACC-SINV-2026-00001"));
    raw_child_row1.insert("parenttype".into(), json!("Sales Invoice"));
    raw_child_row1.insert("parentfield".into(), json!("items"));
    raw_child_row1.insert("idx".into(), json!(1));
    raw_child_row1.insert("item_code".into(), json!("ARM-CORE-M4"));
    raw_child_row1.insert("qty".into(), json!("10.000000"));
    raw_child_row1.insert("rate".into(), json!("150.050000"));

    let mut tables = HashMap::new();
    tables.insert("tabSales Invoice".into(), (parent_schema.clone(), vec![raw_parent_row1]));
    tables.insert("tabSales Invoice Item".into(), (child_schema.clone(), vec![raw_child_row1]));

    let driver = MockMariaDbDriver { tables };

    // 4. Test keyset chunked streaming ingest
    let schemas = driver.discover_schemas().await.unwrap();
    assert_eq!(schemas.len(), 2);

    let batch_parent = driver
        .fetch_keyset_batch("tabSales Invoice", "name", None, 100)
        .await
        .unwrap();
    assert_eq!(batch_parent.records.len(), 1);
    assert!(!batch_parent.has_more);

    let batch_child = driver
        .fetch_keyset_batch("tabSales Invoice Item", "name", None, 100)
        .await
        .unwrap();
    assert_eq!(batch_child.records.len(), 1);

    // 5. Normalize MySQL data types
    let mut normalized_parents: Vec<Map<String, Value>> = batch_parent
        .records
        .into_iter()
        .map(|r| LiveMariaDbStreamer::normalize_row(&parent_schema, r))
        .collect();

    let normalized_children: Vec<Map<String, Value>> = batch_child
        .records
        .into_iter()
        .map(|r| LiveMariaDbStreamer::normalize_row(&child_schema, r))
        .collect();

    // Verify type normalizations
    assert_eq!(normalized_parents[0]["is_return"], json!(false));
    assert_eq!(normalized_parents[0]["grand_total"], json!("1500.5"));
    assert!(normalized_parents[0]["creation"].as_str().unwrap().contains("2026-10-16T12:30:00"));

    // 6. Test Child nesting into parent document
    LiveMariaDbStreamer::nest_child_records(&mut normalized_parents, &normalized_children, "Sales Invoice");
    assert!(normalized_parents[0].contains_key("items"));
    let items_arr = normalized_parents[0]["items"].as_array().unwrap();
    assert_eq!(items_arr.len(), 1);
    assert_eq!(items_arr[0]["item_code"], "ARM-CORE-M4");

    // 7. Comprehensive Side-by-Side Reconciliation Audit
    // Setup GL Entry records for MariaDB source and SurrealDB target
    let gl_row_src1 = json!({
        "account": "Debtors - RustNext",
        "debit": 1500.50,
        "credit": 0.0,
    });
    let gl_row_src2 = json!({
        "account": "Sales - RustNext",
        "debit": 0.0,
        "credit": 1500.50,
    });

    let gl_row_tgt1 = json!({
        "account": "Debtors - RustNext",
        "debit": "1500.50",
        "credit": "0.00",
    });
    let gl_row_tgt2 = json!({
        "account": "Sales - RustNext",
        "debit": "0.00",
        "credit": "1500.50",
    });

    // Setup Stock Ledger records
    let sle_src = json!({
        "item_code": "ARM-CORE-M4",
        "warehouse": "Stores - RN",
        "actual_qty": 50.0,
        "stock_value_difference": 7500.0,
    });
    let sle_tgt = json!({
        "item_code": "ARM-CORE-M4",
        "warehouse": "Stores - RN",
        "actual_qty": "50.0",
        "stock_value_difference": "7500.00",
    });

    let mut source_dataset = HashMap::new();
    source_dataset.insert("GL Entry".into(), vec![gl_row_src1, gl_row_src2]);
    source_dataset.insert("Stock Ledger Entry".into(), vec![sle_src]);
    source_dataset.insert("Sales Invoice".into(), vec![Value::Object(normalized_parents[0].clone())]);

    let mut target_dataset = HashMap::new();
    target_dataset.insert("GL Entry".into(), vec![gl_row_tgt1, gl_row_tgt2]);
    target_dataset.insert("Stock Ledger Entry".into(), vec![sle_tgt]);
    target_dataset.insert("Sales Invoice".into(), vec![Value::Object(normalized_parents[0].clone())]);

    let report = ReconciliationAuditor::execute_audit(&source_dataset, &target_dataset);
    println!("AUDIT REPORT: {:#?}", report);

    assert!(report.is_passed);
    assert_eq!(report.discrepancies.len(), 0);
    assert_eq!(report.gl_dr_source, dec!(1500.50));
    assert_eq!(report.gl_dr_target, dec!(1500.50));
    assert_eq!(report.gl_cr_source, dec!(1500.50));
    assert_eq!(report.gl_cr_target, dec!(1500.50));
    assert_eq!(report.stock_qty_source, dec!(50.0));
    assert_eq!(report.stock_qty_target, dec!(50.0));
    assert_eq!(report.stock_val_source, dec!(7500.0));
    assert_eq!(report.stock_val_target, dec!(7500.0));
    assert_eq!(report.trial_balance_accounts_verified, 2);
    assert_eq!(report.doctype_hashes_match.get("Sales Invoice"), Some(&true));
}
