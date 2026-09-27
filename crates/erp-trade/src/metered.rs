//! High-Throughput Usage-Based Metered Billing & Rating Engine.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingModel {
    FlatRatePerUnit(Decimal),
    TieredGraduated(Vec<(Decimal, Decimal)>), // Vec<(MaxThreshold, RatePerUnit)>
    VolumeTiered(Vec<(Decimal, Decimal)>),
    OverageIncludedAllowance { allowance: Decimal, overage_rate: Decimal },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageEvent {
    pub account_id: String,
    pub meter_code: String,
    pub units_consumed: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct MeteredRatingEngine;

impl MeteredRatingEngine {
    /// Rates a total volume of usage events under specified rating model.
    pub fn calculate_charge(total_units: Decimal, model: &RatingModel) -> Decimal {
        match model {
            RatingModel::FlatRatePerUnit(rate) => (total_units * rate).round_dp(2),
            RatingModel::OverageIncludedAllowance { allowance, overage_rate } => {
                let overage = (total_units - allowance).max(Decimal::ZERO);
                (overage * overage_rate).round_dp(2)
            }
            RatingModel::TieredGraduated(tiers) => {
                let mut remaining = total_units;
                let mut total_cost = Decimal::ZERO;
                let mut prev_threshold = Decimal::ZERO;

                for (threshold, rate) in tiers {
                    if remaining.is_zero() {
                        break;
                    }
                    let tier_capacity = threshold - prev_threshold;
                    let units_in_tier = remaining.min(tier_capacity);
                    total_cost += units_in_tier * rate;
                    remaining -= units_in_tier;
                    prev_threshold = *threshold;
                }
                if remaining > Decimal::ZERO {
                    if let Some((_, last_rate)) = tiers.last() {
                        total_cost += remaining * last_rate;
                    }
                }
                total_cost.round_dp(2)
            }
            RatingModel::VolumeTiered(tiers) => {
                let mut applicable_rate = tiers.last().map(|t| t.1).unwrap_or(Decimal::ZERO);
                for (threshold, rate) in tiers {
                    if total_units <= *threshold {
                        applicable_rate = *rate;
                        break;
                    }
                }
                (total_units * applicable_rate).round_dp(2)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tiered_graduated_metered_rating() {
        let tiers = RatingModel::TieredGraduated(vec![
            (dec!(1000), dec!(0.10)), // First 1,000 units @ $0.10 = $100
            (dec!(5000), dec!(0.08)), // Next 4,000 units @ $0.08 = $320
            (dec!(10000), dec!(0.05)), // Above 5,000 units @ $0.05
        ]);

        let cost_3000 = MeteredRatingEngine::calculate_charge(dec!(3000), &tiers);
        // 1000 * 0.10 (100) + 2000 * 0.08 (160) = $260.00
        assert_eq!(cost_3000, dec!(260.00));
    }
}
