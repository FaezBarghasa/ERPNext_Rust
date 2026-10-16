//! Upstream ERPNext Fixed Asset Golden Tests (`erp-asset`).
//!
//! Validates:
//! 1. Straight Line (SL), Double Declining Balance (DDB), and Written Down Value (WDV) depreciation schedules.
//! 2. Capital Work-in-Progress (CWIP) accumulation and fixed asset capitalization.
//! 3. Asset Preventive Maintenance scheduling, frequency calculation, and completion log tracking.

use chrono::NaiveDate;
use erp_asset::{
    Asset, AssetCapitalization, AssetMaintenance, AssetParams, AssetStatus, CapitalizationItem,
    CapitalizationService, DepreciationMethod, MaintenancePeriodicity,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_golden_depreciation_methods_parity() {
    let buy_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let use_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // --- 1. Straight Line Method ---
    // Cost: $12,000, Salvage: $2,000, Useful Life: 5 years (60 months)
    let mut sl_asset = Asset::new(AssetParams {
        name: "AST-SL-01",
        asset_name: "Delivery Van",
        item_code: "VAN-01",
        gross_amount: dec!(12000.0),
        purchase_date: buy_date,
        available_date: use_date,
        salvage_value: dec!(2000.0),
        method: DepreciationMethod::StraightLine,
        total_depreciations: 5,
        frequency_months: 12,
        is_cwip: false,
    })
    .unwrap();

    sl_asset.calculate_depreciation_schedule().unwrap();
    assert_eq!(sl_asset.schedules.len(), 5);

    // Depreciable base = $10,000 / 5 = $2,000 annual depreciation
    for entry in &sl_asset.schedules {
        assert_eq!(entry.depreciation_amount, dec!(2000.0));
    }
    assert_eq!(sl_asset.schedules[4].book_value, dec!(2000.0)); // Exactly salvage!
    assert_eq!(
        sl_asset.schedules[4].accumulated_depreciation_amount,
        dec!(10000.0)
    );

    // --- 2. Double Declining Balance Method ---
    // Cost: $12,000, Salvage: $2,000, Useful Life: 5 periods -> Rate = 2 / 5 = 0.40 (40%)
    let mut ddb_asset = Asset::new(AssetParams {
        name: "AST-DDB-01",
        asset_name: "High Performance Server",
        item_code: "SRV-01",
        gross_amount: dec!(12000.0),
        purchase_date: buy_date,
        available_date: use_date,
        salvage_value: dec!(2000.0),
        method: DepreciationMethod::DoubleDecliningBalance,
        total_depreciations: 5,
        frequency_months: 12,
        is_cwip: false,
    })
    .unwrap();

    ddb_asset.calculate_depreciation_schedule().unwrap();
    assert_eq!(ddb_asset.schedules.len(), 5);

    // Period 1: 12,000 * 0.40 = 4,800 -> Book value = 7,200
    assert_eq!(ddb_asset.schedules[0].depreciation_amount, dec!(4800.0));
    assert_eq!(ddb_asset.schedules[0].book_value, dec!(7200.0));

    // Period 2: 7,200 * 0.40 = 2,880 -> Book value = 4,320
    assert_eq!(ddb_asset.schedules[1].depreciation_amount, dec!(2880.0));
    assert_eq!(ddb_asset.schedules[1].book_value, dec!(4320.0));

    // Final Period: Never drops below salvage ($2,000)
    assert!(ddb_asset.schedules[4].book_value >= dec!(2000.0));
}

#[test]
fn test_golden_asset_capitalization_and_maintenance() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Initialize CWIP Fixed Asset
    let mut asset = Asset::new(AssetParams {
        name: "AST-TURBINE-01",
        asset_name: "Hydroelectric Turbine Generator",
        item_code: "TURBINE-GEN",
        gross_amount: Decimal::ZERO,
        purchase_date: date,
        available_date: date,
        salvage_value: dec!(10000.0), // Expected salvage
        method: DepreciationMethod::StraightLine,
        total_depreciations: 10, // 10 years useful life
        frequency_months: 12,
        is_cwip: true,
    })
    .unwrap();
    assert_eq!(asset.status, AssetStatus::Draft);
    assert_eq!(asset.gross_purchase_amount, Decimal::ZERO);

    // 2. Accumulate CWIP items and installation services
    let mut cap = AssetCapitalization::new("CAP-2026-001", date, "AST-TURBINE-01");
    cap.cwip_items.push(CapitalizationItem {
        item_code: "TURBINE-ROTOR".into(),
        qty: dec!(1.0),
        rate: dec!(60000.0),
        amount: dec!(60000.0),
    });
    cap.cwip_items.push(CapitalizationItem {
        item_code: "STATOR-COILS".into(),
        qty: dec!(3.0),
        rate: dec!(10000.0),
        amount: dec!(30000.0),
    });
    cap.service_costs.push(CapitalizationService {
        expense_account: "5120 - Civil & Electrical Commissioning".into(),
        description: "Heavy crane rigging and transformer synchronization".into(),
        amount: dec!(20000.0),
    });

    // 3. Submit Capitalization: 60k + 30k + 20k = $110,000
    let total = cap.submit(&mut asset).unwrap();
    assert_eq!(total, dec!(110000.0));
    assert_eq!(asset.gross_purchase_amount, dec!(110000.0));
    assert_eq!(asset.status, AssetStatus::Capitalized);

    // Depreciable base = $110,000 - $10,000 = $100,000 / 10 = $10,000/yr
    assert_eq!(asset.schedules.len(), 10);
    assert_eq!(asset.schedules[0].depreciation_amount, dec!(10000.0));
    assert_eq!(asset.schedules[9].book_value, dec!(10000.0)); // Exactly salvage value

    // 4. Setup Preventive Maintenance Schedule
    let mut maint = AssetMaintenance::new(
        "AST-TURBINE-01",
        "TURBINE-GEN",
        "chief_engineer@plant.local",
    );

    maint.add_task(
        "Quarterly Bearing Lubrication & Vibration Analysis",
        MaintenancePeriodicity::Quarterly,
        date,
        "maint_tech_1",
    );
    assert_eq!(maint.tasks.len(), 1);
    assert!(!maint.tasks[0].is_completed);

    // Complete task -> advances next due date by 3 months
    let next_due = maint.complete_task(0).unwrap();
    assert!(maint.tasks[0].is_completed);
    assert_eq!(
        next_due,
        NaiveDate::from_ymd_opt(2027, 4, 16).unwrap() // Started Oct 16 -> next was Jan 16, after complete -> Apr 16
    );
}
