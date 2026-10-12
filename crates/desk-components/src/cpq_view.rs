//! Interactive CPQ Constraint Visualizer & Margin Waterfall Model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionCard {
    pub option_id: String,
    pub name: String,
    pub group: String,
    pub price: Decimal,
    pub is_selected: bool,
    pub is_disabled: bool,
    pub disable_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpqConfiguratorModel {
    pub quote_id: String,
    pub options: Vec<OptionCard>,
    pub list_price: Decimal,
    pub net_price: Decimal,
    pub pocket_price: Decimal,
    pub gross_margin_percent: Decimal,
    pub requires_executive_approval: bool,
}

impl CpqConfiguratorModel {
    #[must_use]
    pub fn new(quote_id: String) -> Self {
        Self {
            quote_id,
            options: Vec::new(),
            list_price: Decimal::ZERO,
            net_price: Decimal::ZERO,
            pocket_price: Decimal::ZERO,
            gross_margin_percent: Decimal::ZERO,
            requires_executive_approval: false,
        }
    }
}
