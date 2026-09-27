//! Quad-Dimensional Matrix Architecture (WBS, OBS, CBS, RBS).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WbsNode {
    pub id: String,
    pub code: String, // e.g., "1.2.4.1"
    pub name: String,
    pub parent_id: Option<String>,
    pub obs_department_id: String,
    pub cbs_cost_code: String,
    pub rbs_resource_pool_id: String,
    pub planned_cost: Decimal,
    pub actual_cost: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObsNode {
    pub id: String,
    pub department: String,
    pub accountable_person: String,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CbsNode {
    pub cost_code: String, // e.g. "5000-LABOR-DIRECT"
    pub description: String,
    pub account_type: String, // Capitalized, OpEx, Direct Labor, Equipment
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RbsNode {
    pub resource_id: String,
    pub resource_type: String, // Tooling, Skilled Labor, Heavy Machinery
    pub hourly_rate: Decimal,
    pub capacity_hours_per_day: f64,
}

pub struct QuadMatrix {
    pub wbs: Vec<WbsNode>,
    pub obs: Vec<ObsNode>,
    pub cbs: Vec<CbsNode>,
    pub rbs: Vec<RbsNode>,
}

impl QuadMatrix {
    #[must_use]
    pub fn new() -> Self {
        Self {
            wbs: Vec::new(),
            obs: Vec::new(),
            cbs: Vec::new(),
            rbs: Vec::new(),
        }
    }

    pub fn rollup_wbs_costs(&self, node_id: &str) -> (Decimal, Decimal) {
        let mut total_planned = Decimal::ZERO;
        let mut total_actual = Decimal::ZERO;

        for node in &self.wbs {
            if node.id == node_id || node.parent_id.as_deref() == Some(node_id) {
                total_planned += node.planned_cost;
                total_actual += node.actual_cost;
            }
        }
        (total_planned, total_actual)
    }
}

impl Default for QuadMatrix {
    fn default() -> Self {
        Self::new()
    }
}
