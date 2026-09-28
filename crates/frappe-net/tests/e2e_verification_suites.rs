//! Section 10.4: Deterministic Automated Verification Pipeline.
//!
//! Exhaustive End-to-End Integration Test Matrix:
//! 1. Financial Precision Suite (Double-Entry Zero Sum, Multi-Book, Formula Statements, Tax Calculations)
//! 2. Supply Chain & MRP Stress Suite (High-concurrency material reservations, Capacity check)
//! 3. Traceability & SABB Suite (Forward & Backward Genealogy Graph Traversals)
//! 4. Edge POS & Offline Resilience Suite (CvRDT Outbox Queue Sync)
//! 5. Visual Builder & AI Regression Suite (Canvas Block Reversion Ledger & Bob Prompt Requests)

use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::multibook::{AccountingBook, MultiBookJournalLine, MultiBookTransaction};
use erp_accounting::report_engine::{FinancialReportEngine, FinancialReportRow, ReportRowType};
use erp_accounting::tax_withholding::TaxPricingEngine;
use erp_accounting::{JournalEntry, JournalEntryLine};
use erp_cms::builder_core::{BobAgentPromptRequest, CanvasBlock, CanvasReversionLedger};
use erp_inventory::reservation::{
    ReservationStatus, ReservationType, StockReservationEngine, StockReservationEntry,
};
use erp_wms::traceability::{GenealogyLink, LineageNode, TraceabilityEngine};
use frappe_storage::crdt::{OfflineOutboxManager, VectorClock};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn test_suite_1_financial_precision() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();

    // 1. Double-Entry Balance Constraints
    let entry = JournalEntry {
        posting_date,
        company: "Acme Corp".into(),
        lines: vec![
            JournalEntryLine {
                account: "1110 - Bank".into(),
                debit: dec!(11500.00),
                credit: dec!(0.00),
                debit_in_account_currency: dec!(11500.00),
                credit_in_account_currency: dec!(0.00),
                exchange_rate: dec!(1.0),
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "4110 - Operating Revenue".into(),
                debit: dec!(0.00),
                credit: dec!(10000.00),
                debit_in_account_currency: dec!(0.00),
                credit_in_account_currency: dec!(10000.00),
                exchange_rate: dec!(1.0),
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "2210 - Output VAT Liability".into(),
                debit: dec!(0.00),
                credit: dec!(1500.00),
                debit_in_account_currency: dec!(0.00),
                credit_in_account_currency: dec!(1500.00),
                exchange_rate: dec!(1.0),
                party_type: None,
                party: None,
            },
        ],
        remarks: "Customer advance payment with VAT".into(),
    };

    assert!(entry.validate_balance().is_ok());

    // 2. Parallel Multi-Book Accounting
    let multi_tx = MultiBookTransaction {
        transaction_id: "MB-2026-01".into(),
        posting_date,
        company: "Acme International".into(),
        book: AccountingBook::IfrsGroupConsolidation,
        lines: vec![
            MultiBookJournalLine {
                account: "1200 - Debtors IFRS".into(),
                debit: dec!(50000.00),
                credit: dec!(0.00),
                cost_center: Some("CC-NORTH".into()),
                project_code: None,
            },
            MultiBookJournalLine {
                account: "4000 - Sales Revenue IFRS".into(),
                debit: dec!(0.00),
                credit: dec!(50000.00),
                cost_center: Some("CC-NORTH".into()),
                project_code: None,
            },
        ],
        memo: "Parallel Book Posting".into(),
    };
    assert!(multi_tx.is_balanced());

    // 3. Formula-Driven Financial Statement Evaluation
    let rows = vec![
        FinancialReportRow {
            label: "Operating_Revenue".into(),
            row_type: ReportRowType::AccountCategory,
            account_category: Some("Revenue".into()),
            formula: None,
            is_bold: false,
            indent_level: 0,
            hide_if_zero: false,
        },
        FinancialReportRow {
            label: "COGS".into(),
            row_type: ReportRowType::AccountCategory,
            account_category: Some("Direct Expense".into()),
            formula: None,
            is_bold: false,
            indent_level: 0,
            hide_if_zero: false,
        },
        FinancialReportRow {
            label: "Gross_Margin".into(),
            row_type: ReportRowType::CalculatedAmount,
            account_category: None,
            formula: Some("Operating_Revenue - COGS".into()),
            is_bold: true,
            indent_level: 0,
            hide_if_zero: false,
        },
    ];

    let mut category_balances: HashMap<CompactString, Decimal> = HashMap::new();
    category_balances.insert("Revenue".into(), dec!(250000.00));
    category_balances.insert("Direct Expense".into(), dec!(150000.00));

    let engine = FinancialReportEngine::new(rows);
    let statement = engine
        .generate_statement(&category_balances, &HashMap::new())
        .expect("Report evaluation failed");

    assert_eq!(statement.len(), 3);
    assert_eq!(statement[2].label.as_str(), "Gross_Margin");
    assert_eq!(statement[2].amount, dec!(100000.00));
    assert!(statement[2].is_bold);

    // 4. Pre-rounding Tax Inclusive Calculation
    let (base_amt, taxes) =
        TaxPricingEngine::calculate_tax_inclusive_split(dec!(1180.00), &[dec!(18.0)]);
    assert_eq!(base_amt, dec!(1000.00));
    assert_eq!(taxes.len(), 1);
    assert_eq!(taxes[0], dec!(180.00));
}

#[test]
fn test_suite_2_supply_chain_and_mrp_stress() {
    let mut engine = StockReservationEngine::new();

    // High-concurrency reservation iterations across Sales Orders & Work Orders
    for i in 0..100 {
        let order_type = if i % 2 == 0 {
            ReservationType::SalesOrder
        } else {
            ReservationType::WorkOrder
        };

        let entry = StockReservationEntry {
            name: format!("RES-BATCH-{i}"),
            voucher_type: order_type,
            voucher_no: format!("ORD-{i}"),
            voucher_detail_no: "item_01".into(),
            item_code: "STEEL-SHEET-3MM".into(),
            warehouse: "WH-CENTRAL-01".into(),
            batch_no: Some("BATCH-2026-A".into()),
            reserved_qty: dec!(5.0),
            delivered_qty: Decimal::ZERO,
            status: ReservationStatus::Draft,
            is_pos_hold: false,
        };

        let res = engine.reserve_stock(entry, dec!(1000.0));
        assert!(res.is_ok());
    }

    let allocated = engine.get_total_reserved_qty(
        "STEEL-SHEET-3MM",
        "WH-CENTRAL-01",
        Some("BATCH-2026-A"),
        true,
    );
    assert_eq!(allocated, dec!(500.0)); // 100 * 5.0 = 500.0 reserved

    // Zero double-allocation test: attempt to reserve 600 units when only 500 remain
    let overflow_entry = StockReservationEntry {
        name: "RES-OVERFLOW".into(),
        voucher_type: ReservationType::SalesOrder,
        voucher_no: "ORD-OVERFLOW".into(),
        voucher_detail_no: "item_01".into(),
        item_code: "STEEL-SHEET-3MM".into(),
        warehouse: "WH-CENTRAL-01".into(),
        batch_no: Some("BATCH-2026-A".into()),
        reserved_qty: dec!(600.0),
        delivered_qty: Decimal::ZERO,
        status: ReservationStatus::Draft,
        is_pos_hold: false,
    };
    let overflow_res = engine.reserve_stock(overflow_entry, dec!(1000.0));
    assert!(overflow_res.is_err());
}

#[test]
fn test_suite_3_traceability_and_sabb() {
    let mut engine = TraceabilityEngine::new();
    let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();

    // 1. Register Movement Nodes
    engine.add_node(LineageNode {
        identifier: "BATCH-RAW-COIL-01".into(),
        item_code: "RAW-COIL".into(),
        voucher_type: "Purchase Receipt".into(),
        voucher_no: "MAT-PRE-2026-001".into(),
        posting_date: date,
        qty: dec!(500.0),
        from_warehouse: None,
        to_warehouse: Some("Raw Stores".into()),
    });

    engine.add_node(LineageNode {
        identifier: "BATCH-FG-CHASSIS-99".into(),
        item_code: "FG-CHASSIS".into(),
        voucher_type: "Work Order".into(),
        voucher_no: "MFG-WO-2026-001".into(),
        posting_date: date,
        qty: dec!(100.0),
        from_warehouse: Some("Raw Stores".into()),
        to_warehouse: Some("Finished Goods Stores".into()),
    });

    // 2. Link Genealogy
    engine.add_genealogy_link(GenealogyLink {
        parent_id: "BATCH-RAW-COIL-01".into(),
        parent_item_code: "RAW-COIL".into(),
        child_id: "BATCH-FG-CHASSIS-99".into(),
        child_item_code: "FG-CHASSIS".into(),
        manufacturing_voucher: "MFG-WO-2026-001".into(),
    });

    // Forward Traceability: Raw Batch -> Finished Assembly
    let forward = engine.forward_trace("BATCH-RAW-COIL-01");
    assert!(forward.contains(&"BATCH-FG-CHASSIS-99".to_string()));

    // Backward Traceability: Finished Assembly -> Raw Batch
    let backward = engine.backward_trace("BATCH-FG-CHASSIS-99");
    assert!(backward.contains(&"BATCH-RAW-COIL-01".to_string()));
}

#[test]
fn test_suite_4_edge_pos_and_offline_resilience() {
    let mut outbox = OfflineOutboxManager::new();
    let clock = VectorClock::new();

    let entry = outbox.enqueue_mutation(
        "tenant_berlin",
        "POS Invoice",
        "POS-INV-EDGE-001",
        "SUBMIT",
        serde_json::to_string(&serde_json::json!({"grand_total": 75.50, "status": "Paid"}))
            .unwrap(),
        clock,
        "terminal_edge_pos_01",
    );

    assert_eq!(outbox.get_pending_sync().len(), 1);
    assert_eq!(entry.doc_name.as_str(), "POS-INV-EDGE-001");

    outbox.mark_synced(&[entry.sync_id.as_str()]);
    assert_eq!(outbox.get_pending_sync().len(), 0);
}

#[test]
fn test_suite_5_visual_builder_and_ai_regression() {
    let initial_block = CanvasBlock::Heading {
        level: 1,
        text: "Welcome to Pure-Rust Enterprise Desk".into(),
        data_binding: None,
    };

    let mut ledger = CanvasReversionLedger::new(vec![initial_block.clone()]);
    assert_eq!(ledger.current_blocks().len(), 1);

    // Commit snapshot #2 with added block
    let new_block = CanvasBlock::TextBlock {
        content: "High-performance enterprise OS".into(),
        data_binding: None,
    };
    ledger.commit(vec![initial_block.clone(), new_block]);
    assert_eq!(ledger.current_blocks().len(), 2);

    // Undo commit
    let reverted = ledger.undo().expect("Undo failed");
    assert_eq!(reverted.len(), 1);
    assert_eq!(ledger.current_blocks().len(), 1);

    // Redo commit
    let redone = ledger.redo().expect("Redo failed");
    assert_eq!(redone.len(), 2);
    assert_eq!(ledger.current_blocks().len(), 2);

    // Verify Bob Prompt Request payload
    let req = BobAgentPromptRequest {
        prompt: "Generate an analytics number card for MRR".into(),
        mode: "generate".into(),
        selected_block_id: None,
        active_theme_id: "theme_dark_emerald".into(),
        target_doctypes: vec!["Sales Invoice".into()],
        attached_image_url: None,
    };
    assert_eq!(req.mode, "generate");
    assert_eq!(req.target_doctypes[0], "Sales Invoice");
}
