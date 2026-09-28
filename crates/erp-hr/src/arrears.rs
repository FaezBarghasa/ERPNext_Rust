use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Record of retroactive salary difference across a past payroll month.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetroactiveMonthDifference {
    /// Payroll month identifier (e.g. "2026-07").
    pub month: String,
    /// Previously paid gross salary.
    pub previously_paid_gross: Decimal,
    /// Revised gross salary according to new retroactive salary structure.
    pub revised_gross: Decimal,
    /// Gross difference payable (`revised_gross - previously_paid_gross`).
    pub gross_difference: Decimal,
    /// Additional tax withheld on the difference.
    pub additional_tax: Decimal,
    /// Net arrears payable for this month.
    pub net_arrears: Decimal,
}

/// Cumulative Retroactive Salary Arrears calculation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetroactiveArrearsSummary {
    /// Employee identifier.
    pub employee_id: String,
    /// Breakdown per past affected month.
    pub monthly_breakdowns: Vec<RetroactiveMonthDifference>,
    /// Total cumulative gross arrears.
    pub total_gross_arrears: Decimal,
    /// Total cumulative additional tax.
    pub total_tax: Decimal,
    /// Total cumulative net arrears payable.
    pub total_net_arrears: Decimal,
}

/// Retroactive Arrears & LWP Reversal Engine.
pub struct ArrearsEngine;

impl ArrearsEngine {
    /// Computes retroactive arrears across multiple past closed payroll months.
    pub fn calculate_arrears(
        employee_id: &str,
        past_months: &[(String, Decimal, Decimal, Decimal)], // (month, old_gross, new_gross, tax_rate_percent)
    ) -> RetroactiveArrearsSummary {
        let mut breakdowns = Vec::new();
        let mut total_gross_arrears = Decimal::ZERO;
        let mut total_tax = Decimal::ZERO;
        let mut total_net_arrears = Decimal::ZERO;

        for (month, old_gross, new_gross, tax_rate) in past_months {
            let diff = (*new_gross - *old_gross).max(Decimal::ZERO);
            let tax = diff * (*tax_rate / Decimal::from(100));
            let net = diff - tax;

            total_gross_arrears += diff;
            total_tax += tax;
            total_net_arrears += net;

            breakdowns.push(RetroactiveMonthDifference {
                month: month.clone(),
                previously_paid_gross: *old_gross,
                revised_gross: *new_gross,
                gross_difference: diff,
                additional_tax: tax,
                net_arrears: net,
            });
        }

        RetroactiveArrearsSummary {
            employee_id: employee_id.to_string(),
            monthly_breakdowns: breakdowns,
            total_gross_arrears,
            total_tax,
            total_net_arrears,
        }
    }

    /// Reverses Leave Without Pay (LWP) deductions when an absence is regularized into paid leave post-payroll.
    pub fn compute_lwp_reversal(
        daily_rate: Decimal,
        lwp_days_to_reverse: Decimal,
        tax_rate_percent: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        let gross_refund = daily_rate * lwp_days_to_reverse;
        let tax = gross_refund * (tax_rate_percent / Decimal::from(100));
        let net_refund = gross_refund - tax;
        (gross_refund, tax, net_refund)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_retroactive_arrears_calculation() {
        let past_months = vec![
            ("2026-07".to_string(), dec!(4000.0), dec!(4500.0), dec!(10.0)),
            ("2026-08".to_string(), dec!(4000.0), dec!(4500.0), dec!(10.0)),
        ];

        let summary = ArrearsEngine::calculate_arrears("EMP-001", &past_months);

        assert_eq!(summary.total_gross_arrears, dec!(1000.0)); // 500 + 500
        assert_eq!(summary.total_tax, dec!(100.0)); // 10%
        assert_eq!(summary.total_net_arrears, dec!(900.0));
    }

    #[test]
    fn test_lwp_reversal() {
        // Daily rate $150, 2 days LWP reversed, 10% tax
        let (gross, tax, net) = ArrearsEngine::compute_lwp_reversal(dec!(150.0), dec!(2.0), dec!(10.0));
        assert_eq!(gross, dec!(300.0));
        assert_eq!(tax, dec!(30.0));
        assert_eq!(net, dec!(270.0));
    }
}
