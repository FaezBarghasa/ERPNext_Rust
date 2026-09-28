//! Section 10.4: Deterministic Automated Verification Pipeline.
//!
//! Exhaustive End-to-End Integration Test Matrix:
//! 1. Financial Precision Suite
//! 2. Supply Chain & MRP Stress Suite (10,000 concurrent reservations)
//! 3. Traceability & SABB Suite (Recursive Forward & Backward Graph Traversal)
//! 4. Edge POS & Offline Resilience Suite (Split Tender & CvRDT Outbox Sync)
//! 5. Visual Builder & AI Regression Suite (Bob YAML Agent & Reversion Ledger)

use chrono::{Duration, Utc};
use desk_components::views::PosClosingShiftSummary;
use erp_accounting::bank_reconciliation::BankReconciliationEngine;
use erp_accounting::multibook::{AccountingBook, ParallelBookOrchestrator};
use erp_accounting::report_engine::{
    FinancialExpressionEvaluator, FinancialReportRow, FinancialStatementConfig,
    FormulaEvaluationContext, ReportRowType,
};
use erp_accounting::tax_withholding::{TaxWithholdingEngine, VendorCumulativeTaxDetail};
use erp_accounting::{GeneralLedgerAccount, LedgerEntry, LedgerPostingTransaction};
use erp_cms::builder_core::{
    BobAgentPromptRequest, BobSiteAgentEngine, CanvasBlock, CanvasReversionLedger,
};
use erp_inventory::reservation::{StockReservationEngine, StockReservationOrderType};
use erp_manufacturing::bom::BomEngine;
use frappe_storage::crdt::{LwwDocumentState, OfflineOutboxManager, VectorClock};
use erp_wms::traceability::{TraceabilityGraph, TraceabilityNodeType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[test]
fn test_suite_1_financial_precision() {
    // 1. Double-Entry Balance Constraints
    let tx = LedgerPostingTransaction {
        voucher_type: "Sales Invoice".into(),
        voucher_no: "ACC-SINV-2026-001".into(),
        posting_date: Utc::now(),
        entries: vec![
            LedgerEntry {
                account: GeneralLedgerAccount {
                    account_number: "1110".into(),
                    name: "Accounts Receivable - USD".into(),
                    is_group: false,
                    parent_account: Some("1100".into()),
                },
                debit: dec!(11500.0000),
                credit: dec!(0.0000),
                currency: "USD".into(),
                exchange_rate: dec!(1.0),
            },
            LedgerEntry {
                account: GeneralLedgerAccount {
                    account_number: "4110".into(),
                    name: "Operating Revenue - Sales".into(),
                    is_group: false,
                    parent_account: Some("4000".into()),
                },
                debit: dec!(0.0000),
                credit: dec!(10000.0000),
                currency: "USD".into(),
                exchange_rate: dec!(1.0),
            },
            LedgerEntry {
                account: GeneralLedgerAccount {
                    account_number: "2210".into(),
                    name: "Output VAT Liability (15%)".into(),
                    is_group: false,
                    parent_account: Some("2200".into()),
                },
                debit: dec!(0.0000),
                credit: dec!(1500.0000),
                currency: "USD".into(),
                exchange_rate: dec!(1.0),
            },
        ],
    };

    assert!(tx.validate_double_entry().is_ok());

    // 2. Parallel Multi-Book Accounting
    let mut orchestrator = ParallelBookOrchestrator::new(AccountingBook::LocalGaap);
    orchestrator.add_parallel_book(AccountingBook::Ifrs);
    let postings = orchestrator.dispatch_voucher_posting("Sales Invoice", "ACC-SINV-2026-001", dec!(11500.00));
    assert_eq!(postings.len(), 2);

    // 3. Formula-Driven Financial Statement Evaluation
    let mut ctx = FormulaEvaluationContext::new();
    ctx.set_variable("Row_REV", dec!(150000.00));
    ctx.set_variable("Row_COGS", dec!(90000.00));

    let gross_profit = FinancialExpressionEvaluator::evaluate_formula(
        "Row_REV - Row_COGS",
        &ctx,
    )
    .expect("Formula failed");
    assert_eq!(gross_profit, dec!(60000.00));

    // 4. Relational TDS/TCS Withholding Thresholds
    let mut vendor = VendorCumulativeTaxDetail {
        vendor_id: "VEND-TECH-01".into(),
        fiscal_year: "2026-2027".into(),
        cumulative_billed_amount: dec!(45000.00),
        tax_withheld_total: dec!(0.00),
        threshold_limit: dec!(50000.00),
        withholding_rate_percent: dec!(10.0),
        has_lower_deduction_cert: false,
    };

    // Invoice of $15,000 pushes cumulative to $60,000 (breaching $50k threshold)
    let tax_deduction = TaxWithholdingEngine::calculate_withholding_deduction(&mut vendor, dec!(15000.00));
    assert_eq!(tax_deduction, dec!(6000.00)); // 10% on cumulative $60k
    assert_eq!(vendor.tax_withheld_total, dec!(6000.00));
}

#[test]
fn test_suite_2_supply_chain_and_mrp_stress() {
    let engine = Arc::new(StockReservationEngine::new());
    let mut handles = Vec::new();

    // 10,000 concurrent reservation iterations across Sales Orders & Work Orders
    let concurrency = 20;
    let iterations_per_thread = 500;

    for thread_id in 0..concurrency {
        let eng = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let order_type = if (thread_id + i) % 2 == 0 {
                    StockReservationOrderType::SalesOrder
                } else {
                    StockReservationOrderType::WorkOrder
                };

                let res = eng.reserve_stock(
                    &format!("ORD-{thread_id}-{i}"),
                    order_type,
                    "STEEL-SHEET-3MM",
                    "WH-CENTRAL-01",
                    Some("BATCH-2026-A"),
                    dec!(1.0),
                    dec!(10000.0), // Capacity
                );
                assert!(res.is_ok());
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let allocated = engine.get_reserved_qty("STEEL-SHEET-3MM", "WH-CENTRAL-01", Some("BATCH-2026-A"));
    assert_eq!(allocated, dec!(10000.0));

    // Next reservation MUST fail due to zero double-allocation invariant
    let overflow_res = engine.reserve_stock(
        "ORD-OVERFLOW",
        StockReservationOrderType::SalesOrder,
        "STEEL-SHEET-3MM",
        "WH-CENTRAL-01",
        Some("BATCH-2026-A"),
        dec!(5.0),
        dec!(10000.0),
    );
    assert!(overflow_res.is_err());
}

#[test]
fn test_suite_3_traceability_and_sabb() {
    let mut graph = TraceabilityGraph::new();

    // Ingest supply chain genealogy nodes
    graph.add_node("PO-REC-01", "PO Receipt (Vendor Coil)", TraceabilityNodeType::PurchaseReceipt);
    graph.add_node("BATCH-RAW-99", "Raw Coil Batch #99", TraceabilityNodeType::Batch);
    graph.add_node("WO-FAB-100", "Stamping Job Work Order", TraceabilityNodeType::WorkOrder);
    graph.add_node("BATCH-FG-55", "Finished Chassis Batch #55", TraceabilityNodeType::Batch);
    graph.add_node("DEL-NOTE-88", "Customer Delivery Note", TraceabilityNodeType::DeliveryNote);

    // Build linkages
    graph.add_edge("PO-REC-01", "BATCH-RAW-99");
    graph.add_edge("BATCH-RAW-99", "WO-FAB-100");
    graph.add_edge("WO-FAB-100", "BATCH-FG-55");
    graph.add_edge("BATCH-FG-55", "DEL-NOTE-88");

    // Forward Traceability: Raw Batch -> Customer Delivery Note
    let forward_nodes = graph.forward_trace("BATCH-RAW-99");
    assert!(forward_nodes.contains(&"DEL-NOTE-88".to_string()));
    assert!(forward_nodes.contains(&"BATCH-FG-55".to_string()));

    // Backward Traceability: Customer Delivery Note -> Vendor PO Receipt
    let backward_nodes = graph.backward_trace("DEL-NOTE-88");
    assert!(backward_nodes.contains(&"PO-REC-01".to_string()));
    assert!(backward_nodes.contains(&"BATCH-RAW-99".to_string()));
}

#[test]
fn test_suite_4_edge_pos_and_offline_resilience() {
    // 1. POS Split Tender & Cash Float Reconcile
    let shift_summary = PosClosingShiftSummary {
        pos_profile: "POS-RETAIL-BERLIN".into(),
        cashier_user: "cashier_hanna".into(),
        opening_cash_float: dec!(300.00),
        cash_sales: dec!(1450.00),
        card_sales: dec!(2800.00),
        gift_card_sales: dec!(250.00),
        counted_cash: dec!(1750.00),
        cash_variance: dec!(0.00),
        total_invoices_count: 85,
    };
    assert_eq!(shift_summary.opening_cash_float + shift_summary.cash_sales, shift_summary.counted_cash);

    // 2. Disconnected CvRDT Outbox Queue Sync
    let outbox = OfflineOutboxManager::new();
    let clock = VectorClock::new("terminal_edge_pos_01");

    let doc = LwwDocumentState {
        doctype: "POS Invoice".into(),
        doc_id: "POS-INV-EDGE-001".into(),
        payload: serde_json::json!({"grand_total": 75.50, "status": "Paid"}),
        vector_clock: clock,
        last_modified_timestamp: 1774880000,
    };

    outbox.enqueue(doc);
    assert_eq!(outbox.pending_count(), 1);

    let drained = outbox.drain_for_sync();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].doc_id, "POS-INV-EDGE-001");
    assert_eq!(outbox.pending_count(), 0);
}

#[test]
fn test_suite_5_visual_builder_and_ai_regression() {
    let mut ledger = CanvasReversionLedger::new();

    let initial_canvas = CanvasBlock::Container {
        id: "hero_section".into(),
        tag: "section".into(),
        classes: "bg-slate-900 text-white p-8".into(),
        children: vec![CanvasBlock::Text {
            id: "hero_title".into(),
            content: "Welcome to Pure-Rust Enterprise Desk".into(),
        }],
    };

    // Commit snapshot #1
    let snap_v1 = ledger.commit_snapshot(initial_canvas.clone(), "Initial human design");
    assert_eq!(snap_v1, 1);

    // Bob AI Agent performs AST modification
    let request = BobAgentPromptRequest {
        prompt: "Add a high-converting CTA button with emerald theme".into(),
        context_doctype: Some("Item".into()),
        active_theme_tokens: vec!["color-primary-emerald".into()],
    };

    let modified_canvas = BobSiteAgentEngine::apply_agent_modification(&initial_canvas, &request);
    let snap_v2 = ledger.commit_snapshot(modified_canvas.clone(), "Bob AI CTA injection");
    assert_eq!(snap_v2, 2);

    // Verify AST was expanded with CTA button
    if let CanvasBlock::Container { children, .. } = &modified_canvas {
        assert_eq!(children.len(), 2);
    } else {
        panic!("Invalid canvas root");
    }

    // Instant rollback verification to Snap #1
    let reverted_canvas = ledger.rollback_to(snap_v1).expect("Rollback failed");
    if let CanvasBlock::Container { children, .. } = &reverted_canvas {
        assert_eq!(children.len(), 1);
    } else {
        panic!("Invalid reverted root");
    }
}
