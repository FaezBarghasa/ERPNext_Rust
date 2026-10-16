//! Upstream ERPNext Work Order & Job Card Engine (`erp-manufacturing::work_order`).
//!
//! Implements canonical shop-floor manufacturing execution:
//! - Work Order status lifecycle: Draft -> Submitted -> NotStarted -> InProcess -> Completed -> Stopped
//! - BOM component explosion to required raw material lines with scrap calculation
//! - Material transfer to Work-in-Progress (WIP) warehouse
//! - Workstation Job Card generation, operator time tracking, and operational cost calculation
//! - Finished Goods completion entry and stock ledger synchronization.

use chrono::NaiveDateTime;
use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::bom::{Bom, ManufacturingError};

/// Lifecycle state of a Work Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkOrderStatus {
    #[default]
    Draft,
    Submitted,
    NotStarted,
    InProcess,
    Completed,
    Stopped,
    Cancelled,
}

/// Operational state of an individual workstation Job Card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum JobCardStatus {
    #[default]
    Open,
    WorkInProgress,
    Completed,
    Cancelled,
}

/// Raw material requirement line for a Work Order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrderItem {
    pub item_code: CompactString,
    pub source_warehouse: CompactString,
    pub required_qty: Decimal,
    pub transferred_qty: Decimal,
    pub consumed_qty: Decimal,
}

/// Operational workstation routing step for a Work Order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrderOperation {
    pub operation: CompactString,
    pub workstation: CompactString,
    pub planned_time_mins: Decimal,
    pub hour_rate: Decimal,
    pub completed_qty: Decimal,
    pub actual_operating_cost: Decimal,
}

/// Operator time log on a workstation Job Card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobCardTimeLog {
    pub from_time: NaiveDateTime,
    pub to_time: NaiveDateTime,
    pub time_in_mins: Decimal,
    pub employee: CompactString,
}

/// Workstation Job Card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobCard {
    pub job_card_id: CompactString,
    pub work_order: CompactString,
    pub operation: CompactString,
    pub workstation: CompactString,
    pub for_quantity: Decimal,
    pub completed_qty: Decimal,
    pub status: JobCardStatus,
    pub time_logs: Vec<JobCardTimeLog>,
    pub hour_rate: Decimal,
}

impl JobCard {
    /// Records work progress and completed quantity on this workstation operation.
    pub fn record_time_log(
        &mut self,
        from_time: NaiveDateTime,
        to_time: NaiveDateTime,
        employee: impl Into<CompactString>,
        completed_qty: Decimal,
    ) {
        let duration_secs = (to_time - from_time).num_seconds();
        let mins = Decimal::from(duration_secs) / dec!(60.0);

        self.time_logs.push(JobCardTimeLog {
            from_time,
            to_time,
            time_in_mins: mins,
            employee: employee.into(),
        });

        self.completed_qty += completed_qty;
        if self.completed_qty >= self.for_quantity {
            self.status = JobCardStatus::Completed;
        } else {
            self.status = JobCardStatus::WorkInProgress;
        }
    }

    /// Computes total actual operating cost for this Job Card based on logged operator runtime.
    #[must_use]
    pub fn calculate_operating_cost(&self) -> Decimal {
        let total_mins: Decimal = self.time_logs.iter().map(|log| log.time_in_mins).sum();
        (total_mins / dec!(60.0)) * self.hour_rate
    }
}

/// Work Order document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrder {
    pub name: CompactString,
    pub production_item: CompactString,
    pub bom_no: CompactString,
    pub qty: Decimal,
    pub produced_qty: Decimal,
    pub status: WorkOrderStatus,
    pub wip_warehouse: CompactString,
    pub fg_warehouse: CompactString,
    pub required_items: Vec<WorkOrderItem>,
    pub operations: Vec<WorkOrderOperation>,
}

impl WorkOrder {
    /// Creates a new Work Order exploded directly from an active Bill of Materials.
    pub fn from_bom(
        name: impl Into<CompactString>,
        bom: &Bom,
        qty: Decimal,
        wip_warehouse: impl Into<CompactString>,
        fg_warehouse: impl Into<CompactString>,
    ) -> Self {
        let wip_wh: CompactString = wip_warehouse.into();
        let fg_wh: CompactString = fg_warehouse.into();

        // Explode required raw materials including scrap percentage
        let required_items = bom
            .items
            .iter()
            .map(|item| {
                let scrap_multiplier = dec!(1.0) + (item.scrap_percentage / dec!(100.0));
                let needed = (item.qty * qty * scrap_multiplier).round_dp(4);

                WorkOrderItem {
                    item_code: item.item_code.clone().into(),
                    source_warehouse: wip_wh.clone(),
                    required_qty: needed,
                    transferred_qty: Decimal::ZERO,
                    consumed_qty: Decimal::ZERO,
                }
            })
            .collect();

        // Explode operations
        let operations = bom
            .operations
            .iter()
            .map(|op| WorkOrderOperation {
                operation: op.operation.clone().into(),
                workstation: op.workstation.clone().into(),
                planned_time_mins: op.time_in_mins * qty,
                hour_rate: op.hour_rate,
                completed_qty: Decimal::ZERO,
                actual_operating_cost: Decimal::ZERO,
            })
            .collect();

        Self {
            name: name.into(),
            production_item: bom.item.clone().into(),
            bom_no: bom.name.clone().into(),
            qty,
            produced_qty: Decimal::ZERO,
            status: WorkOrderStatus::Draft,
            wip_warehouse: wip_wh,
            fg_warehouse: fg_wh,
            required_items,
            operations,
        }
    }

    /// Submits the Work Order.
    pub fn submit(&mut self) -> Result<(), ManufacturingError> {
        if self.status != WorkOrderStatus::Draft {
            return Err(ManufacturingError::BomNotFound(format!(
                "Cannot submit Work Order in state {:?}",
                self.status
            )));
        }
        self.status = WorkOrderStatus::Submitted;
        Ok(())
    }

    /// Generates Job Cards for each routing operation.
    #[must_use]
    pub fn make_job_cards(&self) -> Vec<JobCard> {
        self.operations
            .iter()
            .enumerate()
            .map(|(i, op)| JobCard {
                job_card_id: format!("{}-JC-{:02}", self.name, i + 1).into(),
                work_order: self.name.clone(),
                operation: op.operation.clone(),
                workstation: op.workstation.clone(),
                for_quantity: self.qty,
                completed_qty: Decimal::ZERO,
                status: JobCardStatus::Open,
                time_logs: Vec::new(),
                hour_rate: op.hour_rate,
            })
            .collect()
    }

    /// Transfers required raw materials from Stores to WIP warehouse.
    pub fn transfer_materials_to_wip(
        &mut self,
        item_code: &str,
        qty: Decimal,
    ) -> Result<(), ManufacturingError> {
        if let Some(item) = self
            .required_items
            .iter_mut()
            .find(|i| i.item_code == item_code)
        {
            item.transferred_qty += qty;
            if self.status == WorkOrderStatus::Submitted {
                self.status = WorkOrderStatus::NotStarted;
            }
            Ok(())
        } else {
            Err(ManufacturingError::BomNotFound(format!(
                "Item '{item_code}' is not a required material for Work Order '{}'",
                self.name
            )))
        }
    }

    /// Records finished good manufacturing receipt into Finished Goods warehouse.
    pub fn record_manufacture_entry(
        &mut self,
        produced_qty: Decimal,
    ) -> Result<(), ManufacturingError> {
        if self.status == WorkOrderStatus::Draft || self.status == WorkOrderStatus::Cancelled {
            return Err(ManufacturingError::BomNotFound(format!(
                "Cannot manufacture in state {:?}",
                self.status
            )));
        }

        // Pro-rate raw material consumption
        let completion_fraction = produced_qty / self.qty;
        for item in &mut self.required_items {
            let consume = (item.required_qty * completion_fraction).round_dp(4);
            item.consumed_qty += consume;
        }

        self.produced_qty += produced_qty;

        if self.produced_qty >= self.qty {
            self.status = WorkOrderStatus::Completed;
        } else {
            self.status = WorkOrderStatus::InProcess;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bom::{BomItem, BomOperation};
    use chrono::NaiveDate;

    #[test]
    fn test_work_order_full_manufacturing_lifecycle() {
        // Setup BOM for DRONE-01
        let bom = Bom {
            name: "BOM-DRONE-01".into(),
            item: "DRONE-01".into(),
            is_phantom: false,
            items: vec![
                BomItem {
                    item_code: "CARBON-FRAME".into(),
                    qty: dec!(1.0),
                    scrap_percentage: dec!(0.0),
                    bom_no: None,
                    is_secondary: false,
                },
                BomItem {
                    item_code: "BLDC-MOTOR".into(),
                    qty: dec!(4.0),
                    scrap_percentage: dec!(5.0), // 5% scrap
                    bom_no: None,
                    is_secondary: false,
                },
            ],
            operations: vec![BomOperation {
                operation: "Airframe Assembly".into(),
                workstation: "BENCH-01".into(),
                time_in_mins: dec!(60.0),
                hour_rate: dec!(40.0),
            }],
            is_active: true,
        };

        // Create Work Order for 10 units
        let mut wo = WorkOrder::from_bom(
            "WO-2026-0001",
            &bom,
            dec!(10.0),
            "Work in Progress - CORP",
            "Finished Goods - CORP",
        );

        assert_eq!(wo.status, WorkOrderStatus::Draft);
        // Carbon frame: 10 * 1 = 10
        assert_eq!(wo.required_items[0].required_qty, dec!(10.0));
        // Motor: 10 * 4 * 1.05 = 42 units
        assert_eq!(wo.required_items[1].required_qty, dec!(42.0));

        wo.submit().unwrap();
        assert_eq!(wo.status, WorkOrderStatus::Submitted);

        // Generate Job Cards
        let mut job_cards = wo.make_job_cards();
        assert_eq!(job_cards.len(), 1);
        assert_eq!(job_cards[0].for_quantity, dec!(10.0));

        // Transfer materials
        wo.transfer_materials_to_wip("CARBON-FRAME", dec!(10.0))
            .unwrap();
        wo.transfer_materials_to_wip("BLDC-MOTOR", dec!(42.0))
            .unwrap();
        assert_eq!(wo.status, WorkOrderStatus::NotStarted);

        // Operator logs time on Job Card
        let t1 = NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap();
        let t2 = NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(13, 0, 0)
            .unwrap(); // 5 hours (300 mins)

        job_cards[0].record_time_log(t1, t2, "operator_1", dec!(10.0));
        assert_eq!(job_cards[0].status, JobCardStatus::Completed);
        // Cost: 5 hours * $40 = $200
        assert_eq!(job_cards[0].calculate_operating_cost(), dec!(200.0));

        // Manufacture Finished Goods (full receipt of 10 units)
        wo.record_manufacture_entry(dec!(10.0)).unwrap();
        assert_eq!(wo.status, WorkOrderStatus::Completed);
        assert_eq!(wo.produced_qty, dec!(10.0));
        assert_eq!(wo.required_items[0].consumed_qty, dec!(10.0));
        assert_eq!(wo.required_items[1].consumed_qty, dec!(42.0));
    }
}
