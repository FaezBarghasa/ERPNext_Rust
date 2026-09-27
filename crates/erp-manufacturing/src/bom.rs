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
    #[error("Workstation '{workstation}' conflict between [{start_1}, {end_1}] and [{start_2}, {end_2}]")]
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
    /// Component lines.
    pub items: Vec<BomItem>,
    /// Routing operations.
    pub operations: Vec<BomOperation>,
    /// Is this the active default BOM?
    pub is_active: bool,
}

/// Graph-Based BOM Engine (Milestones 3.1 & 3.2).
pub struct BomEngine;

impl BomEngine {
    /// Detects circular dependencies across BOM hierarchies using DFS cycle detection.
    pub fn detect_cycles(boms_by_item: &HashMap<String, Bom>) -> Result<(), ManufacturingError> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for item_code in boms_by_item.keys() {
            if !visited.contains(item_code) {
                Self::dfs_cycle_check(item_code, boms_by_item, &mut visited, &mut recursion_stack)?;
            }
        }
        Ok(())
    }

    fn dfs_cycle_check(
        current_item: &str,
        boms_by_item: &HashMap<String, Bom>,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Result<(), ManufacturingError> {
        visited.insert(current_item.to_string());
        recursion_stack.insert(current_item.to_string());

        if let Some(bom) = boms_by_item.get(current_item) {
            for child in &bom.items {
                if !visited.contains(&child.item_code) {
                    Self::dfs_cycle_check(
                        &child.item_code,
                        boms_by_item,
                        visited,
                        recursion_stack,
                    )?;
                } else if recursion_stack.contains(&child.item_code) {
                    return Err(ManufacturingError::CircularDependencyDetected(format!(
                        "Cycle detected: {current_item} -> {}",
                        child.item_code
                    )));
                }
            }
        }

        recursion_stack.remove(current_item);
        Ok(())
    }

    /// Recursive Multi-Level BOM Cost Rollup Engine (Milestone 3.2).
    /// Sums raw material unit costs + workstation operating runtimes across all nesting depths.
    pub fn calculate_cost_rollup(
        item_code: &str,
        boms_by_item: &HashMap<String, Bom>,
        item_valuation_rates: &HashMap<String, Decimal>,
    ) -> Result<Decimal, ManufacturingError> {
        let mut total_cost = Decimal::ZERO;

        if let Some(bom) = boms_by_item.get(item_code) {
            // 1. Operations cost on this assembly
            for op in &bom.operations {
                total_cost += op.calculate_cost();
            }

            // 2. Recursive component cost rollup
            for component in &bom.items {
                let scrap_factor = Decimal::ONE + (component.scrap_percentage / Decimal::from(100));
                let effective_qty = component.qty * scrap_factor;

                let component_unit_cost = if boms_by_item.contains_key(&component.item_code) {
                    // Sub-assembly: recurse
                    Self::calculate_cost_rollup(
                        &component.item_code,
                        boms_by_item,
                        item_valuation_rates,
                    )?
                } else {
                    // Leaf raw material: lookup valuation rate
                    *item_valuation_rates
                        .get(&component.item_code)
                        .unwrap_or(&Decimal::ZERO)
                };

                total_cost += component_unit_cost * effective_qty;
            }
            Ok(total_cost)
        } else {
            // Leaf item valuation rate
            Ok(*item_valuation_rates
                .get(item_code)
                .unwrap_or(&Decimal::ZERO))
        }
    }
}
