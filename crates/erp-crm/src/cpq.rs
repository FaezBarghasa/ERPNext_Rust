//! Constraint-Satisfaction CPQ (Configure, Price, Quote) Solver & Dynamic EBOM Synthesis.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionConstraint {
    pub selected_option: String,
    pub requires_all: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub max_power_draw_kw: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceWaterfall {
    pub list_price: Decimal,
    pub volume_discount: Decimal,
    pub base_price: Decimal,
    pub contract_tier_discount: Decimal,
    pub net_price: Decimal,
    pub freight_and_tariffs: Decimal,
    pub landed_price: Decimal,
    pub early_pay_discount: Decimal,
    pub pocket_price: Decimal,
    pub target_margin_percent: Decimal,
    pub actual_margin_percent: Decimal,
    pub requires_executive_approval: bool,
}

pub struct CpqSolver {
    pub constraints: Vec<OptionConstraint>,
    pub option_costs: HashMap<String, Decimal>,
}

impl CpqSolver {
    #[must_use]
    pub fn new(constraints: Vec<OptionConstraint>, option_costs: HashMap<String, Decimal>) -> Self {
        Self {
            constraints,
            option_costs,
        }
    }

    /// Prunes unavailable options dynamically given current user selections.
    pub fn prune_options(&self, selected: &[String]) -> HashSet<String> {
        let selected_set: HashSet<String> = selected.iter().cloned().collect();
        let mut disabled = HashSet::new();

        for c in &self.constraints {
            if selected_set.contains(&c.selected_option) {
                for conflict in &c.conflicts_with {
                    disabled.insert(conflict.clone());
                }
            }
        }
        disabled
    }

    /// Evaluates multi-tier price waterfall and margin protection thresholds.
    pub fn calculate_waterfall(
        &self,
        list_price: Decimal,
        volume_disc_pct: Decimal,
        tier_disc_pct: Decimal,
        tariffs: Decimal,
        early_pay_disc_pct: Decimal,
        total_cog: Decimal,
    ) -> PriceWaterfall {
        let volume_discount = (list_price * volume_disc_pct).round_dp(2);
        let base_price = list_price - volume_discount;

        let contract_tier_discount = (base_price * tier_disc_pct).round_dp(2);
        let net_price = base_price - contract_tier_discount;

        let landed_price = net_price + tariffs;
        let early_pay_discount = (landed_price * early_pay_disc_pct).round_dp(2);
        let pocket_price = landed_price - early_pay_discount;

        let margin_pct = if pocket_price.is_zero() {
            Decimal::ZERO
        } else {
            (((pocket_price - total_cog) / pocket_price) * Decimal::from(100)).round_dp(2)
        };

        let target_margin = Decimal::from(25); // 25% target margin
        let requires_approval = margin_pct < target_margin;

        PriceWaterfall {
            list_price,
            volume_discount,
            base_price,
            contract_tier_discount,
            net_price,
            freight_and_tariffs: tariffs,
            landed_price,
            early_pay_discount,
            pocket_price,
            target_margin_percent: target_margin,
            actual_margin_percent: margin_pct,
            requires_executive_approval: requires_approval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_cpq_pruning_and_waterfall() {
        let constraints = vec![OptionConstraint {
            selected_option: "Chassis_HeavyDuty".into(),
            requires_all: vec!["Transmission_Allison".into()],
            conflicts_with: vec!["Transmission_Manual5Spd".into()],
            max_power_draw_kw: Some(850.0),
        }];

        let solver = CpqSolver::new(constraints, HashMap::new());
        let disabled = solver.prune_options(&["Chassis_HeavyDuty".into()]);
        assert!(disabled.contains("Transmission_Manual5Spd"));

        let waterfall = solver.calculate_waterfall(
            dec!(100000),
            dec!(0.10),  // 10% volume discount -> Base 90,000
            dec!(0.05),  // 5% tier discount -> Net 85,500
            dec!(5000),  // Landed 90,500
            dec!(0.02),  // Early pay 2% -> Pocket 88,690
            dec!(70000), // COG -> Margin ~21.07% < 25%
        );

        assert_eq!(waterfall.base_price, dec!(90000.00));
        assert_eq!(waterfall.net_price, dec!(85500.00));
        assert_eq!(waterfall.landed_price, dec!(90500.00));
        assert_eq!(waterfall.pocket_price, dec!(88690.00));
        assert!(waterfall.requires_executive_approval);
    }
}
