pub mod bom;
pub mod capa;
pub mod ebr;
pub mod eco;
pub mod milp_scheduler;
pub mod mrp;
pub mod quad_bom;
pub mod spc;

pub use bom::{Bom, BomEngine, BomItem, BomOperation, ManufacturingError};
pub use capa::{EightDPhase, EightDReport, IshikawaCategory};
pub use ebr::{BatchStepRecord, ElectronicBatchRecord, WitnessSignature};
pub use eco::{DispositionMode, EcoStatus, EngineeringChangeOrder};
pub use milp_scheduler::{ProductionJob, SequenceOptimizer, WorkCenterSchedule};
pub use mrp::{MrpEngine, OrderType, PlannedOrder, ScheduledSlot, WorkstationScheduler};
pub use quad_bom::{BomDivergence, BomLineItem, BomType, QuadBomSynchronizer, StructuredBom};
pub use spc::{ControlLimits, SpcEngine, SpcRuleViolation, SpcSubgroup};

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[test]
    fn test_bom_cycle_detection() {
        let mut boms = HashMap::new();

        // BOM A requires BOM B
        boms.insert(
            "ITEM_A".into(),
            Bom {
                name: "BOM-A".into(),
                item: "ITEM_A".into(),
                items: vec![BomItem {
                    item_code: "ITEM_B".into(),
                    qty: dec!(1.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: Some("BOM-B".into()),
                }],
                operations: vec![],
                is_active: true,
            },
        );

        // BOM B requires BOM C
        boms.insert(
            "ITEM_B".into(),
            Bom {
                name: "BOM-B".into(),
                item: "ITEM_B".into(),
                items: vec![BomItem {
                    item_code: "ITEM_C".into(),
                    qty: dec!(1.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: Some("BOM-C".into()),
                }],
                operations: vec![],
                is_active: true,
            },
        );

        // BOM C requires BOM A -> Creates circular dependency A -> B -> C -> A
        boms.insert(
            "ITEM_C".into(),
            Bom {
                name: "BOM-C".into(),
                item: "ITEM_C".into(),
                items: vec![BomItem {
                    item_code: "ITEM_A".into(),
                    qty: dec!(1.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: Some("BOM-A".into()),
                }],
                operations: vec![],
                is_active: true,
            },
        );

        let res = BomEngine::detect_cycles(&boms);
        assert!(matches!(
            res,
            Err(ManufacturingError::CircularDependencyDetected(_))
        ));
    }

    #[test]
    fn test_recursive_bom_cost_rollup() {
        let mut boms = HashMap::new();
        let mut valuation_rates = HashMap::new();

        // Raw material valuation: RAW_CHIP = $5.00
        valuation_rates.insert("RAW_CHIP".into(), dec!(5.00));

        // Sub-assembly: PCB requires 2 RAW_CHIPs ($10.00) + 30 mins operation @ $20/hr ($10.00) = $20.00
        boms.insert(
            "ASSY_PCB".into(),
            Bom {
                name: "BOM-PCB".into(),
                item: "ASSY_PCB".into(),
                items: vec![BomItem {
                    item_code: "RAW_CHIP".into(),
                    qty: dec!(2.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: None,
                }],
                operations: vec![BomOperation {
                    operation: "SMD Assembly".into(),
                    workstation: "SMD-LINE-1".into(),
                    time_in_mins: dec!(30.0),
                    hour_rate: dec!(20.00),
                }],
                is_active: true,
            },
        );

        // Top-level assembly: DRONE requires 2 ASSY_PCBs ($40.00) + 60 mins operation @ $30/hr ($30.00) = $70.00
        boms.insert(
            "DRONE".into(),
            Bom {
                name: "BOM-DRONE".into(),
                item: "DRONE".into(),
                items: vec![BomItem {
                    item_code: "ASSY_PCB".into(),
                    qty: dec!(2.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: Some("BOM-PCB".into()),
                }],
                operations: vec![BomOperation {
                    operation: "Final Assembly".into(),
                    workstation: "ASSEMBLY-BENCH".into(),
                    time_in_mins: dec!(60.0),
                    hour_rate: dec!(30.00),
                }],
                is_active: true,
            },
        );

        let total_cost = BomEngine::calculate_cost_rollup("DRONE", &boms, &valuation_rates)
            .expect("Cost rollup failed");

        assert_eq!(total_cost, dec!(70.00));
    }

    #[test]
    fn test_mrp_net_requirement_and_workstation_scheduling() {
        // MRP calculation: Gross = 20, Stock = 5, Open PO = 3, Safety = 2 -> Net = 14
        let net = MrpEngine::calculate_net_requirement(dec!(20.0), dec!(5.0), dec!(3.0), dec!(2.0));
        assert_eq!(net, dec!(14.0));

        // Workstation scheduling with collision forward sliding
        let mut scheduler = WorkstationScheduler::new();

        // Job 1: wants [100, 200]
        let slot1 = scheduler.schedule_operation("CNC-1", "JOB-01".into(), 100, 100);
        assert_eq!(slot1.start_time, 100);
        assert_eq!(slot1.end_time, 200);

        // Job 2: also wants [100, 200] -> Should slide to [200, 300]
        let slot2 = scheduler.schedule_operation("CNC-1", "JOB-02".into(), 100, 100);
        assert_eq!(slot2.start_time, 200);
        assert_eq!(slot2.end_time, 300);
    }
}
