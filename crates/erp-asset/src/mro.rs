//! Tool Crib & MRO Inventory Optimization using Poisson Failure Probability Distribution.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MroPartRequirement {
    pub part_id: String,
    pub name: String,
    pub failure_rate_per_year: f64, // lambda
    pub lead_time_days: f64,        // t
    pub target_service_level: f64,  // e.g. 0.95 (95% stockout prevention)
}

pub struct PoissonMroOptimizer;

impl PoissonMroOptimizer {
    /// Computes factorial k!
    fn factorial(n: u64) -> f64 {
        if n <= 1 {
            1.0
        } else {
            (1..=n).fold(1.0, |acc, x| acc * x as f64)
        }
    }

    /// Computes Poisson probability P(k failures) = ((lambda*t)^k * e^(-lambda*t)) / k!
    #[must_use]
    pub fn poisson_probability(lambda_t: f64, k: u64) -> f64 {
        if lambda_t <= 0.0 {
            return if k == 0 { 1.0 } else { 0.0 };
        }
        let term = lambda_t.powi(k as i32);
        let exp_term = (-lambda_t).exp();
        (term * exp_term) / Self::factorial(k)
    }

    /// Calculates required safety stock quantity S such that cumulative sum P(k <= S) >= target_service_level.
    #[must_use]
    pub fn recommended_safety_stock(req: &MroPartRequirement) -> u64 {
        let t_years = req.lead_time_days / 365.0;
        let lambda_t = req.failure_rate_per_year * t_years;

        let mut cumulative_prob = 0.0;
        let mut stock: u64 = 0;

        while stock < 1000 {
            cumulative_prob += Self::poisson_probability(lambda_t, stock);
            if cumulative_prob >= req.target_service_level {
                return stock;
            }
            stock += 1;
        }
        stock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poisson_safety_stock_recommendation() {
        let part = MroPartRequirement {
            part_id: "BEARING-SKF-6204".into(),
            name: "High Temp Ceramic Bearing".into(),
            failure_rate_per_year: 12.0, // Expected 1 failure per month
            lead_time_days: 60.0,        // 2 months lead time -> lambda*t = 2.0
            target_service_level: 0.95,  // 95% protection
        };

        let safety_stock = PoissonMroOptimizer::recommended_safety_stock(&part);
        assert!(safety_stock >= 4 && safety_stock <= 6);
    }
}
