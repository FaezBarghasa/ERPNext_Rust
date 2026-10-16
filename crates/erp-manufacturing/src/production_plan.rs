//! Upstream ERPNext Production Plan Engine (`erp-manufacturing::production_plan`).
//!
//! Aggregates sales order demands, explodes multi-level BOM requirements,
//! and orchestrates automatic generation of Work Orders and Material Requests.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bom::Bom;
use crate::work_order::WorkOrder;

/// Production Plan lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProductionPlanStatus {
    #[default]
    Draft,
    Submitted,
    InProcess,
    Completed,
    Cancelled,
}

/// An individual finished goods item planned for production.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductionPlanItem {
    pub item_code: CompactString,
    pub bom_no: CompactString,
    pub planned_qty: Decimal,
    pub sales_order: Option<CompactString>,
}

/// Material requirement line calculated from consolidated BOM explosion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterialRequirementItem {
    pub item_code: CompactString,
    pub required_qty: Decimal,
    pub available_qty: Decimal,
    pub projected_qty: Decimal, // available_qty - required_qty (negative means deficit needing purchase)
}

/// Production Plan document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductionPlan {
    pub name: CompactString,
    pub posting_date: NaiveDate,
    pub status: ProductionPlanStatus,
    pub plan_items: Vec<ProductionPlanItem>,
    pub material_requirements: Vec<MaterialRequirementItem>,
}

impl ProductionPlan {
    pub fn new(name: impl Into<CompactString>, posting_date: NaiveDate) -> Self {
        Self {
            name: name.into(),
            posting_date,
            status: ProductionPlanStatus::Draft,
            plan_items: Vec::new(),
            material_requirements: Vec::new(),
        }
    }

    /// Adds a finished good item to the production plan.
    pub fn add_item(
        &mut self,
        item_code: impl Into<CompactString>,
        bom_no: impl Into<CompactString>,
        planned_qty: Decimal,
        sales_order: Option<impl Into<CompactString>>,
    ) {
        self.plan_items.push(ProductionPlanItem {
            item_code: item_code.into(),
            bom_no: bom_no.into(),
            planned_qty,
            sales_order: sales_order.map(Into::into),
        });
    }

    /// Explodes all BOMs in the plan and calculates consolidated raw material requirements
    /// against current warehouse available stock.
    pub fn calculate_material_requirements(
        &mut self,
        boms: &HashMap<CompactString, Bom>,
        stock_balances: &HashMap<CompactString, Decimal>,
    ) {
        let mut aggregated_requirements: HashMap<CompactString, Decimal> = HashMap::new();

        for plan_item in &self.plan_items {
            if let Some(bom) = boms.get(&plan_item.bom_no) {
                for item in &bom.items {
                    let key = CompactString::from(item.item_code.as_str());
                    let needed = item.qty * plan_item.planned_qty;
                    *aggregated_requirements.entry(key).or_insert(Decimal::ZERO) += needed;
                }
            }
        }

        let mut mr_list = Vec::new();
        for (item_code, req_qty) in aggregated_requirements {
            let available = stock_balances
                .get(&item_code)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let projected = available - req_qty;

            mr_list.push(MaterialRequirementItem {
                item_code,
                required_qty: req_qty,
                available_qty: available,
                projected_qty: projected,
            });
        }

        // Sort by item code for determinism
        mr_list.sort_by(|a, b| a.item_code.cmp(&b.item_code));
        self.material_requirements = mr_list;
    }

    /// Automatically generates Work Orders for each planned item using active BOMs.
    pub fn generate_work_orders(
        &self,
        boms: &HashMap<CompactString, Bom>,
        wip_warehouse: &str,
        fg_warehouse: &str,
    ) -> Vec<WorkOrder> {
        self.plan_items
            .iter()
            .enumerate()
            .filter_map(|(i, plan_item)| {
                boms.get(&plan_item.bom_no).map(|bom| {
                    let wo_name = format!("{}-WO-{:02}", self.name, i + 1);
                    WorkOrder::from_bom(
                        wo_name,
                        bom,
                        plan_item.planned_qty,
                        wip_warehouse,
                        fg_warehouse,
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bom::BomItem;
    use rust_decimal_macros::dec;

    #[test]
    fn test_production_plan_material_explosion_and_work_order_generation() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let mut plan = ProductionPlan::new("PLAN-2026-001", date);

        // BOM for Solar Battery
        let mut boms = HashMap::new();
        boms.insert(
            "BOM-BATTERY".into(),
            Bom {
                name: "BOM-BATTERY".into(),
                item: "SOLAR-BATTERY-5KWH".into(),
                is_phantom: false,
                items: vec![
                    BomItem {
                        item_code: "LFP-CELL".into(),
                        qty: dec!(16.0), // 16 cells per battery
                        scrap_percentage: Decimal::ZERO,
                        bom_no: None,
                        is_secondary: false,
                    },
                    BomItem {
                        item_code: "BMS-BOARD".into(),
                        qty: dec!(1.0),
                        scrap_percentage: Decimal::ZERO,
                        bom_no: None,
                        is_secondary: false,
                    },
                ],
                operations: vec![],
                is_active: true,
            },
        );

        // Plan 10 batteries
        plan.add_item("SOLAR-BATTERY-5KWH", "BOM-BATTERY", dec!(10.0), Some("SO-001"));

        // Current stock: 100 LFP-CELL in warehouse, 10 BMS-BOARD
        let mut stock = HashMap::new();
        stock.insert("LFP-CELL".into(), dec!(100.0));
        stock.insert("BMS-BOARD".into(), dec!(10.0));

        plan.calculate_material_requirements(&boms, &stock);

        // Required: 160 LFP cells, available: 100 -> projected = -60 (deficit)
        // Required: 10 BMS boards, available: 10 -> projected = 0 (balanced)
        let bms = plan
            .material_requirements
            .iter()
            .find(|m| m.item_code == "BMS-BOARD")
            .unwrap();
        assert_eq!(bms.required_qty, dec!(10.0));
        assert_eq!(bms.projected_qty, Decimal::ZERO);

        let cells = plan
            .material_requirements
            .iter()
            .find(|m| m.item_code == "LFP-CELL")
            .unwrap();
        assert_eq!(cells.required_qty, dec!(160.0));
        assert_eq!(cells.projected_qty, dec!(-60.0));

        // Generate Work Orders
        let wos = plan.generate_work_orders(&boms, "WIP Stores", "Finished Goods");
        assert_eq!(wos.len(), 1);
        assert_eq!(wos[0].name, "PLAN-2026-001-WO-01");
        assert_eq!(wos[0].qty, dec!(10.0));
        assert_eq!(wos[0].production_item, "SOLAR-BATTERY-5KWH");
    }
}
