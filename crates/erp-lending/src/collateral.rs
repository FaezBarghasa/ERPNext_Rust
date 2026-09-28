use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Pledged collateral asset record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PledgedCollateral {
    /// Collateral asset ID.
    pub collateral_id: String,
    /// Asset description (e.g. "Commercial Real Estate", "Equity Portfolio").
    pub description: String,
    /// Mark-to-market valuation amount.
    pub market_value: Decimal,
    /// Valuation as-of date.
    pub valuation_date: NaiveDate,
}

/// Margin Call notice issued upon loan covenant breach.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarginCallNotice {
    /// Loan account ID.
    pub loan_id: String,
    /// Current outstanding principal balance.
    pub outstanding_balance: Decimal,
    /// Total collateral mark-to-market valuation.
    pub collateral_value: Decimal,
    /// Current calculated Loan-to-Value (LTV) percentage.
    pub current_ltv: Decimal,
    /// Maximum permitted covenant LTV threshold.
    pub covenant_ltv_limit: Decimal,
    /// Required additional collateral deposit or principal paydown to restore covenant.
    pub required_cure_amount: Decimal,
    /// Issue date of notice.
    pub issue_date: NaiveDate,
}

/// Collateral Monitoring & LTV Engine.
pub struct CollateralEngine;

impl CollateralEngine {
    /// Computes the Loan-to-Value (LTV) ratio as a percentage: `(Outstanding Balance / Total Collateral Value) * 100`.
    pub fn calculate_ltv(
        outstanding_balance: Decimal,
        collateral_items: &[PledgedCollateral],
    ) -> Decimal {
        let total_collateral: Decimal = collateral_items.iter().map(|c| c.market_value).sum();
        if total_collateral <= Decimal::ZERO {
            Decimal::from(100) // 100% or unbounded if no collateral
        } else {
            (outstanding_balance / total_collateral) * Decimal::from(100)
        }
    }

    /// Evaluates whether current LTV breaches the agreed covenant threshold and generates a Margin Call if breached.
    pub fn evaluate_margin_call(
        loan_id: &str,
        outstanding_balance: Decimal,
        collateral_items: &[PledgedCollateral],
        covenant_ltv_limit: Decimal,
        as_of_date: NaiveDate,
    ) -> Option<MarginCallNotice> {
        let total_collateral: Decimal = collateral_items.iter().map(|c| c.market_value).sum();
        let current_ltv = Self::calculate_ltv(outstanding_balance, collateral_items);

        if current_ltv > covenant_ltv_limit {
            // Calculate required cure amount to restore LTV back to covenant_ltv_limit
            // (outstanding_balance - cure) / total_collateral = (covenant_ltv_limit / 100)
            // cure = outstanding_balance - (total_collateral * covenant_ltv_limit / 100)
            let max_allowed_balance = total_collateral * (covenant_ltv_limit / Decimal::from(100));
            let required_cure_amount = (outstanding_balance - max_allowed_balance).max(Decimal::ZERO);

            Some(MarginCallNotice {
                loan_id: loan_id.to_string(),
                outstanding_balance,
                collateral_value: total_collateral,
                current_ltv,
                covenant_ltv_limit,
                required_cure_amount,
                issue_date: as_of_date,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_collateral_ltv_and_margin_call() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        let collateral = vec![
            PledgedCollateral {
                collateral_id: "COL-RE-01".into(),
                description: "Industrial Warehouse".into(),
                market_value: dec!(800000.0), // $800k
                valuation_date: date,
            },
            PledgedCollateral {
                collateral_id: "COL-EQ-02".into(),
                description: "Treasury Bonds".into(),
                market_value: dec!(200000.0), // $200k -> Total = $1,000,000
                valuation_date: date,
            },
        ];

        let outstanding_principal = dec!(750000.0); // $750k -> LTV = 75%
        let ltv = CollateralEngine::calculate_ltv(outstanding_principal, &collateral);
        assert_eq!(ltv, dec!(75.0));

        // Covenant threshold is 70% -> Breached (75% > 70%)
        let notice = CollateralEngine::evaluate_margin_call(
            "LN-CORP-001",
            outstanding_principal,
            &collateral,
            dec!(70.0),
            date,
        )
        .expect("Expected margin call notice");

        assert_eq!(notice.current_ltv, dec!(75.0));
        assert_eq!(notice.covenant_ltv_limit, dec!(70.0));
        // Max allowed balance @ 70% of $1,000,000 is $700,000 -> Cure amount is $50,000
        assert_eq!(notice.required_cure_amount, dec!(50000.0));
    }
}
