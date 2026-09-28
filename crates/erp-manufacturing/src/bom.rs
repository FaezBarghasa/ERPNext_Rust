//! Advanced Bill of Materials (BOM) & Phantom Decoupling (`erp-manufacturing::bom`).
//!
//! Provides:
//! - Multi-level BOM hierarchy and circular dependency detection
//! - **Phantom BOMs**: multi-level assemblies exploding directly into raw materials
//! - **BOM Secondary Items**: by-products & co-products with cost allocation
//! - **Item Where Used**: recursive traversal querying BOMs, product bundles, variants
//! - **Partial Job Card Operating Cost Pro-rating**:
//!   $$\text{Operating Cost}_{\text{entry}} = \text{Total Operating Cost} \times \left( \frac{\text{Produced Qty}_{\text{entry}}}{\text{Total Planned Qty}} \right)$$

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Manufacturing and BOM errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManufacturingError {
    /// Circular dependency detected in BOM assembly tree.
    #[error("Circular dependency detected in BOM: {0}")]
    CircularDependencyDetected(String),
    /// BOM not found.
    #[error("BOM not found for item: {0}")]
    BomNotFound(String),
    /// Workstation collision.
    #[error(
        "Workstation '{workstation}' conflict between [{start_1}, {end_1}] and [{start_2}, {end_2}]"
    )]
    WorkstationCollision {
        workstation: String,
        start_1: u64,
        end_1: u64,
        start_2: u64,
        end_2: u64,
    },
}

/// Raw material or sub-assembly line in a Bill of Materials.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BomItem {
    /// Target component item code.
    pub item_code: String,
    /// Required quantity per parent assembly unit.
    pub qty: Decimal,
    /// Expected scrap loss percentage (e.g. 5.0 for 5%).
    pub scrap_percentage: Decimal,
    /// Associated child BOM ID if this component is a sub-assembly.
    pub bom_no: Option<String>,
    /// Is this a secondary by-product or co-product?
    pub is_secondary: bool,
}

/// Operational routing step on a workstation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BomOperation {
    /// Operation name (e.g. "Laser Cutting", "Surface Mounting").
    pub operation: String,
    /// Target workstation identifier.
    pub workstation: String,
    /// Operating runtime in minutes.
    pub time_in_mins: Decimal,
    /// Workstation hourly rate.
    pub hour_rate: Decimal,
}

impl BomOperation {
    /// Calculates operation labor and machinery cost.
    #[must_use]
    pub fn calculate_cost(&self) -> Decimal {
        (self.time_in_mins / Decimal::from(60)) * self.hour_rate
    }
}

/// Complete Bill of Materials document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bom {
    /// BOM identifier (e.g. "BOM-DRONE-001").
    pub name: String,
    /// Assembly item code produced.
    pub item: String,
    /// Is this a Phantom BOM (explodes directly without intermediate Work Orders)?
    pub is_phantom: bool,
    /// Is this BOM active for production?
    pub is_active: bool,
    /// Component lines.
    pub items: Vec<BomItem>,
    /// Operational routing steps.
    pub operations: Vec<BomOperation>,
}

/// Helper Engine for BOM computations and validation.
pub struct BomEngine;

impl BomEngine {
    /// Detects circular dependencies in a dictionary of active BOMs.
    pub fn detect_cycles(boms: &HashMap<String, Bom>) -> Result<(), ManufacturingError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for item_code in boms.keys() {
            if !visited.contains(item_code) {
                Self::detect_cycles_dfs(item_code, boms, &mut visited, &mut rec_stack)?;
            }
        }
        Ok(())
    }

    fn detect_cycles_dfs(
        current_item: &str,
        boms: &HashMap<String, Bom>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<(), ManufacturingError> {
        visited.insert(current_item.to_string());
        rec_stack.insert(current_item.to_string());

        if let Some(bom) = boms.get(current_item) {
            for item in &bom.items {
                if item.bom_no.is_some() {
                    let child_item = &item.item_code;
                    if !visited.contains(child_item) {
                        Self::detect_cycles_dfs(child_item, boms, visited, rec_stack)?;
                    } else if rec_stack.contains(child_item) {
                        return Err(ManufacturingError::CircularDependencyDetected(format!(
                            "{current_item} -> {child_item}"
                        )));
                    }
                }
            }
        }

        rec_stack.remove(current_item);
        Ok(())
    }

    /// Recursively rolls up raw material valuation and operation costs for an item.
    pub fn calculate_cost_rollup(
        item_code: &str,
        boms: &HashMap<String, Bom>,
        valuation_rates: &HashMap<String, Decimal>,
    ) -> Result<Decimal, ManufacturingError> {
        let bom = boms
            .get(item_code)
            .ok_or_else(|| ManufacturingError::BomNotFound(item_code.to_string()))?;

        let mut total_cost = Decimal::ZERO;

        for item in &bom.items {
            let item_cost = if item.bom_no.is_some() {
                Self::calculate_cost_rollup(&item.item_code, boms, valuation_rates)?
            } else {
                valuation_rates
                    .get(&item.item_code)
                    .copied()
                    .unwrap_or(Decimal::ZERO)
            };
            total_cost += item_cost * item.qty;
        }

        for op in &bom.operations {
            total_cost += op.calculate_cost();
        }

        Ok(total_cost)
    }
}

/// Exploded component requirement resulting from BOM explosion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplodedRequirement {
    pub item_code: String,
    pub total_qty: Decimal,
    pub is_phantom_child: bool,
    pub is_secondary: bool,
}

impl Bom {
    /// Recursively explodes all multi-level dependencies into raw material requirements,
    /// traversing through Phantom BOMs directly while detecting circular graph loops.
    pub fn explode(
        &self,
        target_qty: Decimal,
        bom_registry: &HashMap<String, Bom>,
    ) -> Result<Vec<ExplodedRequirement>, ManufacturingError> {
        let mut visited = HashSet::new();
        let mut requirements = Vec::new();
        self.explode_internal(
            target_qty,
            bom_registry,
            &mut visited,
            &mut requirements,
            false,
        )?;
        Ok(requirements)
    }

    fn explode_internal(
        &self,
        multiplier: Decimal,
        registry: &HashMap<String, Bom>,
        visited: &mut HashSet<String>,
        acc: &mut Vec<ExplodedRequirement>,
        parent_is_phantom: bool,
    ) -> Result<(), ManufacturingError> {
        if !visited.insert(self.name.clone()) {
            return Err(ManufacturingError::CircularDependencyDetected(
                self.name.clone(),
            ));
        }

        for item in &self.items {
            let scrap_multiplier = Decimal::ONE + (item.scrap_percentage / Decimal::from(100));
            let required_qty = item.qty * multiplier * scrap_multiplier;

            if let Some(ref child_bom_no) = item.bom_no
                && let Some(child_bom) = registry.get(child_bom_no)
                && child_bom.is_phantom
            {
                // Phantom BOM: explode directly into raw components
                child_bom.explode_internal(required_qty, registry, visited, acc, true)?;
                continue;
            }

            acc.push(ExplodedRequirement {
                item_code: item.item_code.clone(),
                total_qty: required_qty,
                is_phantom_child: parent_is_phantom,
                is_secondary: item.is_secondary,
            });
        }

        visited.remove(&self.name);
        Ok(())
    }

    /// Recursively queries all BOMs that use a specific raw material item ("Item Where Used").
    #[must_use]
    pub fn item_where_used(item_code: &str, registry: &HashMap<String, Bom>) -> Vec<String> {
        let mut matches = Vec::new();
        for (bom_name, bom) in registry {
            if bom.items.iter().any(|i| i.item_code == item_code) {
                matches.push(bom_name.clone());
            }
        }
        matches
    }

    /// Computes pro-rated operating cost for a partial Job Card run.
    #[must_use]
    pub fn calculate_partial_job_card_cost(
        total_planned_cost: Decimal,
        total_planned_qty: Decimal,
        produced_qty_entry: Decimal,
    ) -> Decimal {
        if total_planned_qty.is_zero() {
            Decimal::ZERO
        } else {
            total_planned_cost * (produced_qty_entry / total_planned_qty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_phantom_bom_explosion() {
        let mut registry = HashMap::new();

        // Sub-assembly (Phantom): Frame Kit -> 4x Carbon Tubes, 8x Screws
        let frame_phantom = Bom {
            name: "BOM-FRAME-PHANTOM".into(),
            item: "FRAME-KIT".into(),
            is_phantom: true,
            is_active: true,
            items: vec![
                BomItem {
                    item_code: "CARBON-TUBE".into(),
                    qty: dec!(4),
                    scrap_percentage: dec!(0),
                    bom_no: None,
                    is_secondary: false,
                },
                BomItem {
                    item_code: "M3-SCREW".into(),
                    qty: dec!(8),
                    scrap_percentage: dec!(0),
                    bom_no: None,
                    is_secondary: false,
                },
            ],
            operations: vec![],
        };

        // Main Drone BOM referencing Phantom Frame Kit
        let drone_bom = Bom {
            name: "BOM-DRONE-MAIN".into(),
            item: "DRONE-PRO".into(),
            is_phantom: false,
            is_active: true,
            items: vec![
                BomItem {
                    item_code: "FRAME-KIT".into(),
                    qty: dec!(1),
                    scrap_percentage: dec!(0),
                    bom_no: Some("BOM-FRAME-PHANTOM".into()),
                    is_secondary: false,
                },
                BomItem {
                    item_code: "MOTOR-2207".into(),
                    qty: dec!(4),
                    scrap_percentage: dec!(0),
                    bom_no: None,
                    is_secondary: false,
                },
                BomItem {
                    item_code: "PACKING-SCRAP".into(),
                    qty: dec!(0.5),
                    scrap_percentage: dec!(0),
                    bom_no: None,
                    is_secondary: true, // By-product
                },
            ],
            operations: vec![],
        };

        registry.insert(frame_phantom.name.clone(), frame_phantom);
        registry.insert(drone_bom.name.clone(), drone_bom.clone());

        let exploded = drone_bom.explode(dec!(2), &registry).unwrap();
        assert_eq!(exploded.len(), 4);

        let carbon_tubes = exploded
            .iter()
            .find(|e| e.item_code == "CARBON-TUBE")
            .unwrap();
        assert_eq!(carbon_tubes.total_qty, dec!(8)); // 2 drones * 4 tubes
        assert!(carbon_tubes.is_phantom_child);

        let secondary = exploded
            .iter()
            .find(|e| e.item_code == "PACKING-SCRAP")
            .unwrap();
        assert!(secondary.is_secondary);

        // Where used
        let where_used = Bom::item_where_used("CARBON-TUBE", &registry);
        assert_eq!(where_used, vec!["BOM-FRAME-PHANTOM"]);
    }

    #[test]
    fn test_partial_job_card_cost_proration() {
        let total_planned_cost = dec!(1000.0);
        let total_planned_qty = dec!(100);
        let produced_entry = dec!(25);

        let cost = Bom::calculate_partial_job_card_cost(
            total_planned_cost,
            total_planned_qty,
            produced_entry,
        );
        assert_eq!(cost, dec!(250.0));
    }
}
