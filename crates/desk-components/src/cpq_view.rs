//! Interactive CPQ Constraint Visualizer & Margin Waterfall Model.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionCard {
    pub option_id: String,
    pub name: String,
    pub group: String,
    pub price: f64,
    pub is_selected: bool,
    pub is_disabled: bool,
    pub disable_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpqConfiguratorModel {
    pub quote_id: String,
    pub options: Vec<OptionCard>,
    pub list_price: f64,
    pub net_price: f64,
    pub pocket_price: f64,
    pub gross_margin_percent: f64,
    pub requires_executive_approval: bool,
}

impl CpqConfiguratorModel {
    #[must_use]
    pub fn new(quote_id: String) -> Self {
        Self {
            quote_id,
            options: Vec::new(),
            list_price: 0.0,
            net_price: 0.0,
            pocket_price: 0.0,
            gross_margin_percent: 0.0,
            requires_executive_approval: false,
        }
    }
}
