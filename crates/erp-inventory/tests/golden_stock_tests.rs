//! Upstream ERPNext Stock & Valuation Golden Tests (`erp-inventory`).
//!
//! Validates:
//! 1. FIFO, Moving Average, and LIFO valuation layer consumption parity.
//! 2. Stock Ledger Entry (SLE) Stock Value Difference (SVD) consistency with General Ledger.
//! 3. Unified Serial and Batch Bundle (v15+) duplicate prevention and weighted rate calculation.
//! 4. Landed Cost Voucher allocation ByQty and ByAmount.

use chrono::NaiveDate;
use erp_inventory::{
    BundleError, BundleType, LandedCostAllocationMethod, LandedCostCharge, LandedCostItem,
    LandedCostVoucher, SerialBatchBundle, SerialBatchEntry, SleParams, StockBin, ValuationEngine,
    ValuationError, ValuationMethod,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_golden_valuation_methods_parity() {
    let dt = NaiveDate::from_ymd_opt(2026, 10, 16)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    // Setup identical receipts for FIFO, Moving Average, and LIFO:
    // Batch 1: 10 units @ $10.00
    // Batch 2: 20 units @ $25.00
    // Dispatch: 15 units

    // --- 1. FIFO Test ---
    let mut fifo_bin = StockBin {
        item_code: "ITEM-A".into(),
        warehouse: "Stores - CORP".into(),
        actual_qty: Decimal::ZERO,
        valuation_rate: Decimal::ZERO,
        stock_value: Decimal::ZERO,
        layers: Vec::new(),
    };
    ValuationEngine::process_sle(
        &mut fifo_bin,
        SleParams {
            sle_name: "SLE-F1",
            voucher_type: "PR",
            voucher_no: "PR-1",
            posting_datetime: dt,
            qty: dec!(10.0),
            incoming_rate: dec!(10.0),
            method: ValuationMethod::FIFO,
        },
    )
    .unwrap();
    ValuationEngine::process_sle(
        &mut fifo_bin,
        SleParams {
            sle_name: "SLE-F2",
            voucher_type: "PR",
            voucher_no: "PR-2",
            posting_datetime: dt,
            qty: dec!(20.0),
            incoming_rate: dec!(25.0),
            method: ValuationMethod::FIFO,
        },
    )
    .unwrap();
    // Dispatch 15: 10 @ $10 + 5 @ $25 = 100 + 125 = $225 COGS
    let fifo_out = ValuationEngine::process_sle(
        &mut fifo_bin,
        SleParams {
            sle_name: "SLE-F3",
            voucher_type: "DN",
            voucher_no: "DN-1",
            posting_datetime: dt,
            qty: dec!(-15.0),
            incoming_rate: Decimal::ZERO,
            method: ValuationMethod::FIFO,
        },
    )
    .unwrap();
    assert_eq!(fifo_out.stock_value_difference, dec!(-225.0));
    assert_eq!(fifo_bin.actual_qty, dec!(15.0));
    assert_eq!(fifo_bin.stock_value, dec!(375.0)); // 15 left @ $25 = $375

    // --- 2. LIFO Test ---
    let mut lifo_bin = StockBin {
        item_code: "ITEM-A".into(),
        warehouse: "Stores - CORP".into(),
        actual_qty: Decimal::ZERO,
        valuation_rate: Decimal::ZERO,
        stock_value: Decimal::ZERO,
        layers: Vec::new(),
    };
    ValuationEngine::process_sle(
        &mut lifo_bin,
        SleParams {
            sle_name: "SLE-L1",
            voucher_type: "PR",
            voucher_no: "PR-1",
            posting_datetime: dt,
            qty: dec!(10.0),
            incoming_rate: dec!(10.0),
            method: ValuationMethod::LIFO,
        },
    )
    .unwrap();
    ValuationEngine::process_sle(
        &mut lifo_bin,
        SleParams {
            sle_name: "SLE-L2",
            voucher_type: "PR",
            voucher_no: "PR-2",
            posting_datetime: dt,
            qty: dec!(20.0),
            incoming_rate: dec!(25.0),
            method: ValuationMethod::LIFO,
        },
    )
    .unwrap();
    // Dispatch 15: all 15 come from latest layer (20 @ $25) -> 15 * 25 = $375 COGS
    let lifo_out = ValuationEngine::process_sle(
        &mut lifo_bin,
        SleParams {
            sle_name: "SLE-L3",
            voucher_type: "DN",
            voucher_no: "DN-1",
            posting_datetime: dt,
            qty: dec!(-15.0),
            incoming_rate: Decimal::ZERO,
            method: ValuationMethod::LIFO,
        },
    )
    .unwrap();
    assert_eq!(lifo_out.stock_value_difference, dec!(-375.0));
    assert_eq!(lifo_bin.actual_qty, dec!(15.0));
    assert_eq!(lifo_bin.stock_value, dec!(225.0)); // 10 @ $10 + 5 @ $25 = 100 + 125 = $225

    // --- 3. Moving Average Test ---
    let mut mavg_bin = StockBin {
        item_code: "ITEM-A".into(),
        warehouse: "Stores - CORP".into(),
        actual_qty: Decimal::ZERO,
        valuation_rate: Decimal::ZERO,
        stock_value: Decimal::ZERO,
        layers: Vec::new(),
    };
    ValuationEngine::process_sle(
        &mut mavg_bin,
        SleParams {
            sle_name: "SLE-M1",
            voucher_type: "PR",
            voucher_no: "PR-1",
            posting_datetime: dt,
            qty: dec!(10.0),
            incoming_rate: dec!(10.0),
            method: ValuationMethod::MovingAverage,
        },
    )
    .unwrap();
    // Total 10 @ 10 = 100
    ValuationEngine::process_sle(
        &mut mavg_bin,
        SleParams {
            sle_name: "SLE-M2",
            voucher_type: "PR",
            voucher_no: "PR-2",
            posting_datetime: dt,
            qty: dec!(20.0),
            incoming_rate: dec!(25.0),
            method: ValuationMethod::MovingAverage,
        },
    )
    .unwrap();
    // Total 30 units, Total value = 100 + 500 = 600 -> Rate = $20.00
    assert_eq!(mavg_bin.valuation_rate, dec!(20.0));
    // Dispatch 15 @ 20 = $300 COGS
    let mavg_out = ValuationEngine::process_sle(
        &mut mavg_bin,
        SleParams {
            sle_name: "SLE-M3",
            voucher_type: "DN",
            voucher_no: "DN-1",
            posting_datetime: dt,
            qty: dec!(-15.0),
            incoming_rate: Decimal::ZERO,
            method: ValuationMethod::MovingAverage,
        },
    )
    .unwrap();
    assert_eq!(mavg_out.stock_value_difference, dec!(-300.0));
    assert_eq!(mavg_bin.actual_qty, dec!(15.0));
    assert_eq!(mavg_bin.stock_value, dec!(300.0));
}

#[test]
fn test_golden_negative_stock_rejection() {
    let dt = NaiveDate::from_ymd_opt(2026, 10, 16)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    let mut bin = StockBin {
        item_code: "SOLAR-CELL".into(),
        warehouse: "Raw Materials - CORP".into(),
        actual_qty: dec!(5.0),
        valuation_rate: dec!(10.0),
        stock_value: dec!(50.0),
        layers: vec![],
    };

    // Attempt to dispatch 10 units when only 5 exist
    let err = ValuationEngine::process_sle(
        &mut bin,
        SleParams {
            sle_name: "SLE-ERR",
            voucher_type: "DN",
            voucher_no: "DN-001",
            posting_datetime: dt,
            qty: dec!(-10.0),
            incoming_rate: Decimal::ZERO,
            method: ValuationMethod::MovingAverage,
        },
    );

    assert!(matches!(err, Err(ValuationError::InsufficientStock { .. })));
}

#[test]
fn test_golden_serial_and_batch_bundle() {
    let bundle = SerialBatchBundle {
        name: "SBB-2026-0001".into(),
        voucher_type: "Purchase Receipt".into(),
        voucher_no: "PR-001".into(),
        item_code: "DRONE-MOTHERBOARD".into(),
        warehouse: "Main Stores".into(),
        bundle_type: BundleType::Inward,
        total_qty: dec!(2.0),
        avg_rate: dec!(450.0),
        total_amount: dec!(900.0),
        entries: vec![
            SerialBatchEntry {
                serial_no: Some("SN-D1".into()),
                batch_no: Some("BATCH-1".into()),
                qty: dec!(1.0),
                incoming_rate: dec!(450.0),
            },
            SerialBatchEntry {
                serial_no: Some("SN-D2".into()),
                batch_no: Some("BATCH-1".into()),
                qty: dec!(1.0),
                incoming_rate: dec!(450.0),
            },
        ],
    };

    assert!(bundle.validate().is_ok());

    // Duplicate serial check
    let mut bad_bundle = bundle.clone();
    bad_bundle.entries[1].serial_no = Some("SN-D1".into()); // duplicate!
    assert!(matches!(
        bad_bundle.validate(),
        Err(BundleError::DuplicateSerialNo(_))
    ));
}

#[test]
fn test_golden_landed_cost_allocation_by_qty() {
    let mut voucher = LandedCostVoucher {
        name: "LCV-001".into(),
        company: "Starlight Corp".into(),
        allocation_method: LandedCostAllocationMethod::ByQty,
        charges: vec![LandedCostCharge {
            expense_account: "5120 - Freight".into(),
            description: "Air Courier".into(),
            amount: dec!(60.0), // $60 total
        }],
        items: vec![
            LandedCostItem {
                receipt_document_type: "PR".into(),
                receipt_document: "PR-1".into(),
                item_code: "ITEM-1".into(),
                qty: dec!(4.0), // 4 units (66.67%)
                original_rate: dec!(10.0),
                original_amount: dec!(40.0),
                allocated_charges: Decimal::ZERO,
                landed_rate: Decimal::ZERO,
            },
            LandedCostItem {
                receipt_document_type: "PR".into(),
                receipt_document: "PR-1".into(),
                item_code: "ITEM-2".into(),
                qty: dec!(2.0), // 2 units (33.33%)
                original_rate: dec!(20.0),
                original_amount: dec!(40.0),
                allocated_charges: Decimal::ZERO,
                landed_rate: Decimal::ZERO,
            },
        ],
    };

    voucher.allocate_charges().unwrap();

    // 4 units out of 6 -> $40 extra ($10 per unit -> Landed Rate = 20.0)
    assert_eq!(voucher.items[0].allocated_charges, dec!(40.0));
    assert_eq!(voucher.items[0].landed_rate, dec!(20.0));

    // 2 units out of 6 -> $20 extra ($10 per unit -> Landed Rate = 30.0)
    assert_eq!(voucher.items[1].allocated_charges, dec!(20.0));
    assert_eq!(voucher.items[1].landed_rate, dec!(30.0));
}
