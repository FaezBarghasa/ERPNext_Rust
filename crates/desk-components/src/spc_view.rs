//! Real-Time Streaming SPC Chart Model for Shop Floor Telemetry.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpcPointView {
    pub subgroup_id: usize,
    pub mean: f64,
    pub range: f64,
    pub is_violation: bool,
    pub rule_tag: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpcChartViewModel {
    pub parameter_name: String,
    pub target_value: f64,
    pub grand_mean: f64,
    pub ucl: f64,
    pub lcl: f64,
    pub points: Vec<SpcPointView>,
    pub active_violations_count: usize,
}

impl SpcChartViewModel {
    #[must_use]
    pub fn new(parameter_name: String, target: f64, mean: f64, ucl: f64, lcl: f64) -> Self {
        Self {
            parameter_name,
            target_value: target,
            grand_mean: mean,
            ucl,
            lcl,
            points: Vec::new(),
            active_violations_count: 0,
        }
    }

    pub fn push_point(&mut self, point: SpcPointView) {
        if point.is_violation {
            self.active_violations_count += 1;
        }
        self.points.push(point);
    }
}
