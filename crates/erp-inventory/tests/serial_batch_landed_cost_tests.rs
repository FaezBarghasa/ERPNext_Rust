//! Integration tests verifying Phase 4 Serial Numbers, Batches, and Landed Cost Engine.
//!
//! Validates:
//! 1. Serial Number receipt with duplicate prevention.
//! 2. Serial Number dispatch verifying Active state and source warehouse, transitioning to Delivered.
//! 3. Batch FEFO/FIFO allocation, deducting quantities and rejecting expired batches.
//! 4. Landed Cost Voucher distribution by Quantity and Amount.
//! 5. Landed Cost Engine generating balanced GL entries and valuation layer adjustments.

use chrono::NaiveDate;
use erp_inventory::batches::Batch;
use erp_inventory::landed_cost_engine::LandedCostEngine;
use erp_inventory::landed_cost_voucher::{
    LandedCostAllocationMethod, LandedCostCharge, LandedCostItem, LandedCostVoucher,
};
use erp_inventory::serial_state_machine::{
    SerialLifecycleState, SerialReceiptParams, SerialRegistry, SerialStateError,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_serial_lifecycle_invariants_and_dispatch() {
    let mut registry = SerialRegistry::new();
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Register serial units via Purchase Receipt
    let p1 = SerialReceiptParams {
        serial_no: "SN-TITAN-001".into(),
        item_code: "TITAN-DRONE".into(),
        warehouse: "Stores - WH1".into(),
        voucher_type: "Purchase Receipt".into(),
        voucher_no: "MAT-PR-2026-00001".into(),
        posting_date,
        batch_no: Some("BATCH-2026-A".into()),
    };

    let rec = registry.register_receipt(p1.clone()).unwrap();
    assert_eq!(rec.state, SerialLifecycleState::Active);
    assert_eq!(rec.warehouse.as_deref(), Some("Stores - WH1"));

    // 2. Reject duplicate receipt while still Active
    let err = registry.register_receipt(p1).unwrap_err();
    assert!(matches!(err, SerialStateError::SerialAlreadyActive(..)));

    // 3. Reject dispatch from wrong warehouse
    let dispatch_err = registry
        .deliver("SN-TITAN-001", "Finished Goods - WH2", "Delivery Note".into(), "MAT-DN-001".into(), posting_date)
        .unwrap_err();
    assert!(matches!(dispatch_err, SerialStateError::WarehouseMismatch { .. }));

    // 4. Successful dispatch -> Transitions to Delivered
    let delivered_rec = registry
        .deliver("SN-TITAN-001", "Stores - WH1", "Delivery Note".into(), "MAT-DN-001".into(), posting_date)
        .unwrap();
    assert_eq!(delivered_rec.state, SerialLifecycleState::Delivered);
    assert!(delivered_rec.warehouse.is_none());

    // 5. Reject re-delivering already delivered serial
    let double_deliv_err = registry
        .deliver("SN-TITAN-001", "Stores - WH1", "Delivery Note".into(), "MAT-DN-002".into(), posting_date)
        .unwrap_err();
    assert!(matches!(double_deliv_err, SerialStateError::IllegalTransition { .. }));
}

#[test]
fn test_batch_fefo_allocation_and_expiry_enforcement() {
    let today = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let expired_date = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
    let future_date1 = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
    let future_date2 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

    let batches = vec![
        // Expired batch
        Batch {
            batch_id: "BATCH-EXP".into(),
            item_code: "CATALYST-B".into(),
            mfg_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            expiry_date: Some(expired_date),
            remaining_qty: dec!(50.0),
        },
        // Earlier expiring active batch
        Batch {
            batch_id: "BATCH-NEAR".into(),
            item_code: "CATALYST-B".into(),
            mfg_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            expiry_date: Some(future_date1),
            remaining_qty: dec!(30.0),
        },
        // Later expiring active batch
        Batch {
            batch_id: "BATCH-FAR".into(),
            item_code: "CATALYST-B".into(),
            mfg_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            expiry_date: Some(future_date2),
            remaining_qty: dec!(50.0),
        },
    ];

    // Single batch validation fails on expired
    assert!(batches[0].validate_usable(today).is_err());

    // Test FEFO allocation on unexpired batches
    let mut unexpired_batches = vec![batches[1].clone(), batches[2].clone()];
    let allocations = Batch::select_fifo_batches(&mut unexpired_batches, dec!(45.0), today).unwrap();

    // Should take all 30 from BATCH-NEAR and 15 from BATCH-FAR
    assert_eq!(allocations.len(), 2);
    assert_eq!(allocations[0], ("BATCH-NEAR".to_string(), dec!(30.0)));
    assert_eq!(allocations[1], ("BATCH-FAR".to_string(), dec!(15.0)));
    assert_eq!(unexpired_batches[0].remaining_qty, dec!(0.0));
    assert_eq!(unexpired_batches[1].remaining_qty, dec!(35.0));

    // If an expired batch is included, allocation must fail
    let mut mixed = vec![batches[0].clone(), batches[1].clone()];
    let err = Batch::select_fifo_batches(&mut mixed, dec!(10.0), today);
    assert!(err.is_err());
}

#[test]
fn test_landed_cost_engine_and_gl_balancing() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    let mut voucher = LandedCostVoucher {
        name: "LCV-2026-00001".into(),
        company: "RustNext International".into(),
        allocation_method: LandedCostAllocationMethod::ByAmount,
        charges: vec![
            LandedCostCharge {
                expense_account: "5120 - Customs Duties".into(),
                description: "Border Import Duty".into(),
                amount: dec!(600.0),
            },
            LandedCostCharge {
                expense_account: "5130 - Freight Inward".into(),
                description: "Ocean Container Line".into(),
                amount: dec!(400.0),
            },
        ], // Total charges = $1,000
        items: vec![
            LandedCostItem {
                receipt_document_type: "Purchase Receipt".into(),
                receipt_document: "MAT-PR-001".into(),
                item_code: "HIGH-END-CPU".into(),
                qty: dec!(10.0),
                original_rate: dec!(300.0),
                original_amount: dec!(3000.0), // 75% of value ($3000 / $4000)
                allocated_charges: Decimal::ZERO,
                landed_rate: Decimal::ZERO,
            },
            LandedCostItem {
                receipt_document_type: "Purchase Receipt".into(),
                receipt_document: "MAT-PR-001".into(),
                item_code: "COOLING-FAN".into(),
                qty: dec!(50.0),
                original_rate: dec!(20.0),
                original_amount: dec!(1000.0), // 25% of value ($1000 / $4000)
                allocated_charges: Decimal::ZERO,
                landed_rate: Decimal::ZERO,
            },
        ],
    };

    let outcome = LandedCostEngine::process_landed_cost(&mut voucher, posting_date, "1510 - Stock In Hand").unwrap();

    assert_eq!(outcome.total_allocated_charges, dec!(1000.0));
    assert_eq!(outcome.layer_adjustments[0].additional_allocated_charges, dec!(750.0));
    assert_eq!(outcome.layer_adjustments[0].adjusted_rate, dec!(375.0)); // 300 + 750/10

    assert_eq!(outcome.layer_adjustments[1].additional_allocated_charges, dec!(250.0));
    assert_eq!(outcome.layer_adjustments[1].adjusted_rate, dec!(25.0)); // 20 + 250/50

    // GL Balances Check: Sum(Debits) == Sum(Credits)
    let total_debit: Decimal = outcome.gl_entries.iter().map(|e| e.debit).sum();
    let total_credit: Decimal = outcome.gl_entries.iter().map(|e| e.credit).sum();
    assert_eq!(total_debit, dec!(1000.0));
    assert_eq!(total_credit, dec!(1000.0));
    assert_eq!(outcome.gl_entries[0].account, "1510 - Stock In Hand");
    assert_eq!(outcome.gl_entries[0].debit, dec!(1000.0));
}
