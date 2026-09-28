use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Dynamic Pricing Rule definition with priority-weighted discount matrix (Milestone 2.9).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PricingRule {
    /// Rule name / code.
    pub name: String,
    /// Target customer group (None = all).
    pub customer_group: Option<String>,
    /// Target item group (None = all).
    pub item_group: Option<String>,
    /// Minimum qualifying quantity.
    pub min_qty: Decimal,
    /// Maximum qualifying quantity.
    pub max_qty: Option<Decimal>,
    /// Priority weight (higher priority evaluated first).
    pub priority: u32,
    /// Percentage discount (e.g. 10.0 for 10%).
    pub discount_percentage: Decimal,
    /// Flat rate discount per unit.
    pub discount_amount: Decimal,
}

/// Dynamic Pricing Solver Engine.
pub struct PricingEngine;

impl PricingEngine {
    /// Resolves the applicable unit price by matching quantity and priority.
    #[must_use]
    pub fn resolve_price(
        base_rate: Decimal,
        qty: Decimal,
        customer_group: Option<&str>,
        item_group: Option<&str>,
        rules: &[PricingRule],
    ) -> Decimal {
        let mut applicable_rules: Vec<&PricingRule> = rules
            .iter()
            .filter(|r| {
                if qty < r.min_qty {
                    return false;
                }
                if let Some(max) = r.max_qty
                    && qty > max
                {
                    return false;
                }
                if let Some(cg) = &r.customer_group
                    && customer_group != Some(cg.as_str())
                {
                    return false;
                }
                if let Some(ig) = &r.item_group
                    && item_group != Some(ig.as_str())
                {
                    return false;
                }
                true
            })
            .collect();

        // Sort by priority descending
        applicable_rules.sort_by_key(|a| std::cmp::Reverse(a.priority));

        if let Some(best_rule) = applicable_rules.first() {
            let pct_factor = Decimal::ONE - (best_rule.discount_percentage / Decimal::from(100));
            let discounted = (base_rate * pct_factor) - best_rule.discount_amount;
            discounted.max(Decimal::ZERO)
        } else {
            base_rate
        }
    }
}
