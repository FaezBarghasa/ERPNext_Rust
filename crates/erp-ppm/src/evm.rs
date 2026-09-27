//! ANSI/EIA-748 Earned Value Management (EVM) and cost/schedule forecasting engine.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvmInputs {
    pub budget_at_completion: Decimal,     // BAC
    pub planned_percent_complete: Decimal, // % Planned
    pub actual_percent_complete: Decimal,  // % Actual physical complete
    pub actual_cost: Decimal,              // AC
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvmMetrics {
    pub planned_value: Decimal,                 // PV = BAC * % Planned
    pub earned_value: Decimal,                  // EV = BAC * % Actual
    pub actual_cost: Decimal,                   // AC
    pub cost_variance: Decimal,                 // CV = EV - AC
    pub schedule_variance: Decimal,             // SV = EV - PV
    pub cost_performance_index: Decimal,        // CPI = EV / AC
    pub schedule_performance_index: Decimal,    // SPI = EV / PV
    pub estimate_at_completion: Decimal,        // EAC = AC + (BAC - EV) / (CPI * SPI)
    pub to_complete_performance_index: Decimal, // TCPI = (BAC - EV) / (BAC - AC)
}

pub struct EvmEngine;

impl EvmEngine {
    pub fn compute(inputs: &EvmInputs) -> Result<EvmMetrics, String> {
        let pv = (inputs.budget_at_completion * inputs.planned_percent_complete).round_dp(2);
        let ev = (inputs.budget_at_completion * inputs.actual_percent_complete).round_dp(2);
        let ac = inputs.actual_cost;

        let cv = ev - ac;
        let sv = ev - pv;

        let cpi = if ac.is_zero() {
            Decimal::ONE
        } else {
            (ev / ac).round_dp(4)
        };

        let spi = if pv.is_zero() {
            Decimal::ONE
        } else {
            (ev / pv).round_dp(4)
        };

        let denominator = cpi * spi;
        let eac = if denominator.is_zero() {
            inputs.budget_at_completion
        } else {
            (ac + (inputs.budget_at_completion - ev) / denominator).round_dp(2)
        };

        let remaining_budget = inputs.budget_at_completion - ac;
        let tcpi = if remaining_budget.is_zero() {
            Decimal::ONE
        } else {
            ((inputs.budget_at_completion - ev) / remaining_budget).round_dp(4)
        };

        Ok(EvmMetrics {
            planned_value: pv,
            earned_value: ev,
            actual_cost: ac,
            cost_variance: cv,
            schedule_variance: sv,
            cost_performance_index: cpi,
            schedule_performance_index: spi,
            estimate_at_completion: eac,
            to_complete_performance_index: tcpi,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_evm_metrics_calculation() {
        let inputs = EvmInputs {
            budget_at_completion: dec!(100000),
            planned_percent_complete: dec!(0.50), // PV = 50,000
            actual_percent_complete: dec!(0.40),  // EV = 40,000
            actual_cost: dec!(45000),             // AC = 45,000
        };

        let metrics = EvmEngine::compute(&inputs).unwrap();
        assert_eq!(metrics.planned_value, dec!(50000.00));
        assert_eq!(metrics.earned_value, dec!(40000.00));
        assert_eq!(metrics.cost_variance, dec!(-5000.00));
        assert_eq!(metrics.schedule_variance, dec!(-10000.00));
        assert_eq!(metrics.cost_performance_index, dec!(0.8889));
        assert_eq!(metrics.schedule_performance_index, dec!(0.8000));
        assert!(metrics.estimate_at_completion > dec!(100000));
    }
}
