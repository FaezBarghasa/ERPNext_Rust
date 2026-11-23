//! Integration tests verifying Phase 5 Manufacturing MRP, Multi-Level BOM & Subcontracting.
//!
//! Validates:
//! 1. Multi-level BOM recursive explosion and cost rollup.
//! 2. Work Order shopfloor execution with Job Card time tracking.
//! 3. Work Order backflushing: raw material consumption, finished goods stock intake, and GL balancing.
//! 4. Subcontracting sourcing flow: Material Transfer to Subcontractor and Subcontracted Purchase Receipt with service cost capitalization.

use chrono::NaiveDate;
use erp_manufacturing::backflushing::BackflushController;
use erp_manufacturing::bom::{Bom, BomEngine, BomItem, BomOperation};
use erp_manufacturing::subcontracting::{
    SubcontractingOrder, SubcontractingOrderItem, SubcontractingReceipt,
    SubcontractingReceiptItem, SubcontractingReceiptSuppliedItem, SubcontractingSuppliedItem,
};
use erp_manufacturing::work_order::WorkOrder;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn test_multilevel_bom_explosion_and_cost_rollup() {
    let mut boms = HashMap::new();

    // 1. Sub-assembly BOM: MOTOR-ASM
    let motor_bom = Bom {
        name: "BOM-MOTOR-01".into(),
        item: "MOTOR-ASM".into(),
        is_phantom: false,
        is_active: true,
        items: vec![
            BomItem {
                item_code: "COPPER-WIRE".into(),
                qty: dec!(2.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "MAGNET-NEODYMIUM".into(),
                qty: dec!(4.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
        ],
        operations: vec![BomOperation {
            operation: "Winding & Assembly".into(),
            workstation: "WS-WINDING".into(),
            time_in_mins: dec!(30.0), // 0.5 hour
            hour_rate: dec!(40.0),   // $20 operation cost
        }],
    };

    // 2. Main Assembly BOM: DRONE-X
    let drone_bom = Bom {
        name: "BOM-DRONE-X".into(),
        item: "DRONE-X".into(),
        is_phantom: false,
        is_active: true,
        items: vec![
            BomItem {
                item_code: "CARBON-FRAME".into(),
                qty: dec!(1.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "MOTOR-ASM".into(),
                qty: dec!(4.0),
                scrap_percentage: dec!(0.0),
                bom_no: Some("BOM-MOTOR-01".into()),
                is_secondary: false,
            },
        ],
        operations: vec![BomOperation {
            operation: "Final Drone Integration".into(),
            workstation: "WS-ASSEMBLY".into(),
            time_in_mins: dec!(60.0), // 1.0 hour
            hour_rate: dec!(50.0),   // $50 operation cost
        }],
    };

    boms.insert("MOTOR-ASM".into(), motor_bom.clone());
    boms.insert("DRONE-X".into(), drone_bom.clone());

    // Valuation rates
    let mut rates = HashMap::new();
    rates.insert("COPPER-WIRE".into(), dec!(5.0));      // 2 * 5 = 10
    rates.insert("MAGNET-NEODYMIUM".into(), dec!(2.5)); // 4 * 2.5 = 10
    rates.insert("CARBON-FRAME".into(), dec!(100.0));   // 100

    // Motor cost = (2*5 + 4*2.5) + (30/60 * 40) = 20 + 20 = $40.0
    let motor_cost = BomEngine::calculate_cost_rollup("MOTOR-ASM", &boms, &rates).unwrap();
    assert_eq!(motor_cost, dec!(40.0));

    // Drone cost = (1*100 + 4*40) + (60/60 * 50) = 100 + 160 + 50 = $310.0
    let drone_cost = BomEngine::calculate_cost_rollup("DRONE-X", &boms, &rates).unwrap();
    assert_eq!(drone_cost, dec!(310.0));

    // Circular loop detection
    assert!(BomEngine::detect_cycles(&boms).is_ok());
}

#[test]
fn test_work_order_backflushing_and_stock_intake() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    let bom = Bom {
        name: "BOM-ROBOT-01".into(),
        item: "ROBOT-CORE".into(),
        is_phantom: false,
        is_active: true,
        items: vec![
            BomItem {
                item_code: "ALU-CHASSIS".into(),
                qty: dec!(1.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "MCU-BOARD".into(),
                qty: dec!(2.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
        ],
        operations: vec![BomOperation {
            operation: "CNC & Flashing".into(),
            workstation: "WS-CNC".into(),
            time_in_mins: dec!(60.0),
            hour_rate: dec!(30.0),
        }],
    };

    let mut wo = WorkOrder::from_bom(
        "WO-ROBOT-00001",
        &bom,
        dec!(10.0),
        "WIP - WH",
        "FG - WH",
    );
    wo.submit().unwrap();

    // Raw material rates
    let mut rates = HashMap::new();
    rates.insert("ALU-CHASSIS".into(), dec!(50.0)); // 10 * 50 = 500
    rates.insert("MCU-BOARD".into(), dec!(25.0));   // 20 * 25 = 500
    // Operation cost = 10 * 30 = 300
    // Total production cost = 500 + 500 + 300 = $1300 ($130 / unit)

    let outcome = BackflushController::execute_backflush(
        &mut wo,
        dec!(10.0),
        posting_date,
        &rates,
        "WIP - WH",
        "FG - WH",
        "1520 - Work In Progress",
        "1510 - Finished Goods Inventory",
    )
    .unwrap();

    assert_eq!(outcome.produced_qty, dec!(10.0));
    assert_eq!(outcome.unit_finished_cost, dec!(130.0));
    assert_eq!(outcome.total_production_cost, dec!(1300.0));

    // Verify Stock Ledger Entries (2 raw material consumptions + 1 finished goods receipt)
    assert_eq!(outcome.stock_ledger_entries.len(), 3);
    assert_eq!(outcome.stock_ledger_entries[0].actual_qty, dec!(-10.0));
    assert_eq!(outcome.stock_ledger_entries[1].actual_qty, dec!(-20.0));
    assert_eq!(outcome.stock_ledger_entries[2].actual_qty, dec!(10.0));
    assert_eq!(outcome.stock_ledger_entries[2].valuation_rate, dec!(130.0));

    // Verify GL Balance: Sum(Debits) == Sum(Credits)
    let total_dr: Decimal = outcome.gl_entries.iter().map(|e| e.debit).sum();
    let total_cr: Decimal = outcome.gl_entries.iter().map(|e| e.credit).sum();
    assert_eq!(total_dr, dec!(1300.0));
    assert_eq!(total_cr, dec!(1300.0));
    assert_eq!(outcome.gl_entries[0].account, "1510 - Finished Goods Inventory");
    assert_eq!(outcome.gl_entries[1].account, "1520 - Work In Progress");
}

#[test]
fn test_subcontracting_sourcing_flow() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    let mut sco = SubcontractingOrder {
        name: "SCO-2026-00001".into(),
        supplier: "Foxconn Assembly Ltd".into(),
        posting_date,
        company: "RustNext International".into(),
        items: vec![SubcontractingOrderItem {
            item_code: "PCB-MAIN-ASM".into(),
            qty: dec!(100.0),
            rate: dec!(15.0), // $15 service charge per board
            amount: dec!(1500.0),
            bom_no: Some("BOM-PCB-01".into()),
        }],
        supplied_items: vec![
            SubcontractingSuppliedItem {
                main_item_code: "PCB-MAIN-ASM".into(),
                rm_item_code: "BARE-PCB".into(),
                required_qty: dec!(100.0),
                consumed_qty: Decimal::ZERO,
                rate: dec!(10.0),
                amount: dec!(1000.0),
            },
            SubcontractingSuppliedItem {
                main_item_code: "PCB-MAIN-ASM".into(),
                rm_item_code: "SMD-CAP-PACK".into(),
                required_qty: dec!(200.0),
                consumed_qty: Decimal::ZERO,
                rate: dec!(5.0),
                amount: dec!(1000.0),
            },
        ],
        total_service_cost: Decimal::ZERO,
        docstatus: 0,
        remarks: None,
    };

    sco.validate().unwrap();
    sco.submit().unwrap();
    assert_eq!(sco.total_service_cost, dec!(1500.0));

    // Subcontracting Receipt: Receiving finished goods and consuming supplied components
    let mut scr = SubcontractingReceipt {
        name: "SCR-2026-00001".into(),
        subcontracting_order: "SCO-2026-00001".into(),
        supplier: "Foxconn Assembly Ltd".into(),
        supplier_warehouse: "Subcontractor - Foxconn".into(),
        accepted_warehouse: "Stores - WH".into(),
        posting_date,
        company: "RustNext International".into(),
        items: vec![SubcontractingReceiptItem {
            item_code: "PCB-MAIN-ASM".into(),
            qty: dec!(100.0),
            service_rate: dec!(15.0),
            service_amount: Decimal::ZERO,
            rm_cost_amount: Decimal::ZERO,
            incoming_rate: Decimal::ZERO,
            amount: Decimal::ZERO,
        }],
        supplied_items: vec![
            SubcontractingReceiptSuppliedItem {
                main_item_code: "PCB-MAIN-ASM".into(),
                rm_item_code: "BARE-PCB".into(),
                consumed_qty: dec!(100.0),
                rate: dec!(10.0),
                amount: dec!(1000.0),
            },
            SubcontractingReceiptSuppliedItem {
                main_item_code: "PCB-MAIN-ASM".into(),
                rm_item_code: "SMD-CAP-PACK".into(),
                consumed_qty: dec!(200.0),
                rate: dec!(5.0),
                amount: dec!(1000.0),
            },
        ],
        total_service_cost: Decimal::ZERO,
        total_rm_cost: Decimal::ZERO,
        grand_total: Decimal::ZERO,
        docstatus: 0,
        remarks: None,
    };

    scr.submit().unwrap();
    let sles = scr.generate_stock_ledger_entries();
    let gls = scr.generate_gl_entries("2120 - Subcontracting Accrual", |wh| {
        match wh {
            "Stores - WH" => Ok("1510 - Stock In Hand".into()),
            "Subcontractor - Foxconn" => Ok("1515 - Subcontractor Stock".into()),
            _ => Err(erp_manufacturing::subcontracting::SubcontractingError::WarehouseAccountNotFound(wh.into())),
        }
    }).unwrap();

    // 100 * (15 service + 10 BARE + 10 CAP) = $35 unit rate => $3500 total
    assert_eq!(scr.items[0].incoming_rate, dec!(35.0));
    assert_eq!(scr.grand_total, dec!(3500.0));

    // Verify SLEs (2 consumption + 1 receipt)
    assert_eq!(sles.len(), 3);
    assert_eq!(sles[0].actual_qty, dec!(-100.0));
    assert_eq!(sles[1].actual_qty, dec!(-200.0));
    assert_eq!(sles[2].actual_qty, dec!(100.0));
    assert_eq!(sles[2].incoming_rate, dec!(35.0));

    // Verify GL entries: Total Debits == Total Credits == $3500
    let total_dr: Decimal = gls.iter().map(|e| e.debit).sum();
    let total_cr: Decimal = gls.iter().map(|e| e.credit).sum();
    assert_eq!(total_dr, dec!(3500.0));
    assert_eq!(total_cr, dec!(3500.0));
}
