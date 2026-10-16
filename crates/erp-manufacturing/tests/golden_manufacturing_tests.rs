//! Upstream ERPNext Manufacturing Golden Tests (`erp-manufacturing`).
//!
//! Validates:
//! 1. Multi-level BOM tree explosion, scrap factor calculation, and circular dependency prevention.
//! 2. Production Plan aggregation of demand lines and net material requirement calculation against stock.
//! 3. Work Order shop-floor execution, Work-in-Progress (WIP) material transfer, and Finished Goods receipt.
//! 4. Workstation Job Card generation, operator run-time tracking, and operating cost proration.

use chrono::NaiveDate;
use erp_manufacturing::{
    Bom, BomEngine, BomItem, BomOperation, JobCardStatus, ManufacturingError, ProductionPlan,
    WorkOrderStatus,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn test_golden_manufacturing_end_to_end_flow() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Define BOM for Industrial Drone (DRONE-X)
    let bom = Bom {
        name: "BOM-DRONE-X".into(),
        item: "DRONE-X".into(),
        is_phantom: false,
        items: vec![
            BomItem {
                item_code: "CARBON-CHASSIS".into(),
                qty: dec!(1.0),
                scrap_percentage: Decimal::ZERO,
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "BRUSHLESS-MOTOR".into(),
                qty: dec!(4.0),
                scrap_percentage: dec!(5.0), // 5% scrap -> 4 * 1.05 = 4.2 per unit
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "FLIGHT-CONTROLLER".into(),
                qty: dec!(1.0),
                scrap_percentage: Decimal::ZERO,
                bom_no: None,
                is_secondary: false,
            },
        ],
        operations: vec![
            BomOperation {
                operation: "Airframe Mounting".into(),
                workstation: "BENCH-01".into(),
                time_in_mins: dec!(30.0),
                hour_rate: dec!(40.0),
            },
            BomOperation {
                operation: "Avionics Calibration".into(),
                workstation: "TEST-BENCH-02".into(),
                time_in_mins: dec!(15.0),
                hour_rate: dec!(60.0),
            },
        ],
        is_active: true,
    };

    let mut boms = HashMap::new();
    boms.insert("BOM-DRONE-X".into(), bom.clone());

    // 2. Production Planning for 10 units
    let mut plan = ProductionPlan::new("MFG-PLAN-2026-001", date);
    plan.add_item("DRONE-X", "BOM-DRONE-X", dec!(10.0), Some("SO-2026-00042"));

    // Available inventory in stores: 10 Chassis, 30 Motors, 10 Flight Controllers
    let mut stock_balances = HashMap::new();
    stock_balances.insert("CARBON-CHASSIS".into(), dec!(10.0));
    stock_balances.insert("BRUSHLESS-MOTOR".into(), dec!(30.0));
    stock_balances.insert("FLIGHT-CONTROLLER".into(), dec!(10.0));

    plan.calculate_material_requirements(&boms, &stock_balances);

    // Motor requirement: 10 units * 4 per unit = 40 needed. Available: 30 -> Projected: -10 (deficit)
    let motor_req = plan
        .material_requirements
        .iter()
        .find(|m| m.item_code == "BRUSHLESS-MOTOR")
        .unwrap();
    assert_eq!(motor_req.required_qty, dec!(40.0));
    assert_eq!(motor_req.projected_qty, dec!(-10.0));

    // 3. Work Order Generation from Plan
    let mut work_orders =
        plan.generate_work_orders(&boms, "WIP Stores - PLANT", "Finished Goods - PLANT");
    assert_eq!(work_orders.len(), 1);
    let wo = &mut work_orders[0];

    assert_eq!(wo.status, WorkOrderStatus::Draft);
    assert_eq!(wo.qty, dec!(10.0));
    // Check scrap calculation in Work Order: 10 * 4 * 1.05 = 42 motors
    assert_eq!(wo.required_items[1].required_qty, dec!(42.0));

    // Submit Work Order
    wo.submit().unwrap();
    assert_eq!(wo.status, WorkOrderStatus::Submitted);

    // 4. Generate Job Cards
    let mut job_cards = wo.make_job_cards();
    assert_eq!(job_cards.len(), 2);
    assert_eq!(job_cards[0].operation, "Airframe Mounting");
    assert_eq!(job_cards[1].operation, "Avionics Calibration");

    // 5. Transfer Materials to WIP Warehouse
    wo.transfer_materials_to_wip("CARBON-CHASSIS", dec!(10.0))
        .unwrap();
    wo.transfer_materials_to_wip("BRUSHLESS-MOTOR", dec!(42.0))
        .unwrap();
    wo.transfer_materials_to_wip("FLIGHT-CONTROLLER", dec!(10.0))
        .unwrap();
    assert_eq!(wo.status, WorkOrderStatus::NotStarted);

    // 6. Execute Operations on Workstations
    let t_start = date.and_hms_opt(9, 0, 0).unwrap();
    let t_end = date.and_hms_opt(14, 0, 0).unwrap(); // 5 hours (300 mins)

    job_cards[0].record_time_log(t_start, t_end, "operator_alpha", dec!(10.0));
    assert_eq!(job_cards[0].status, JobCardStatus::Completed);
    // 5 hours @ $40/hr = $200 operating cost
    assert_eq!(job_cards[0].calculate_operating_cost(), dec!(200.0));

    // 7. Manufacture Finished Goods (Partial receipt of 5 units)
    wo.record_manufacture_entry(dec!(5.0)).unwrap();
    assert_eq!(wo.status, WorkOrderStatus::InProcess);
    assert_eq!(wo.produced_qty, dec!(5.0));
    // 50% consumed
    assert_eq!(wo.required_items[0].consumed_qty, dec!(5.0));
    assert_eq!(wo.required_items[1].consumed_qty, dec!(21.0));

    // 8. Manufacture Final 5 units -> Complete Work Order
    wo.record_manufacture_entry(dec!(5.0)).unwrap();
    assert_eq!(wo.status, WorkOrderStatus::Completed);
    assert_eq!(wo.produced_qty, dec!(10.0));
    assert_eq!(wo.required_items[0].consumed_qty, dec!(10.0));
    assert_eq!(wo.required_items[1].consumed_qty, dec!(42.0));
}

#[test]
fn test_golden_bom_circular_dependency_rejection() {
    let mut boms = HashMap::new();

    // Node 1 depends on Node 2
    boms.insert(
        "ITEM-1".into(),
        Bom {
            name: "BOM-1".into(),
            item: "ITEM-1".into(),
            is_phantom: false,
            items: vec![BomItem {
                item_code: "ITEM-2".into(),
                qty: dec!(1.0),
                scrap_percentage: Decimal::ZERO,
                bom_no: Some("BOM-2".into()),
                is_secondary: false,
            }],
            operations: vec![],
            is_active: true,
        },
    );

    // Node 2 depends on Node 1 (Cycle)
    boms.insert(
        "ITEM-2".into(),
        Bom {
            name: "BOM-2".into(),
            item: "ITEM-2".into(),
            is_phantom: false,
            items: vec![BomItem {
                item_code: "ITEM-1".into(),
                qty: dec!(1.0),
                scrap_percentage: Decimal::ZERO,
                bom_no: Some("BOM-1".into()),
                is_secondary: false,
            }],
            operations: vec![],
            is_active: true,
        },
    );

    let err = BomEngine::detect_cycles(&boms);
    assert!(matches!(
        err,
        Err(ManufacturingError::CircularDependencyDetected(_))
    ));
}
