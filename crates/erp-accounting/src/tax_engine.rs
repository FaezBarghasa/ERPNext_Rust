//! Multi-Tier Tax Engine (`erp-accounting::tax_engine`).
//!
//! Evaluates complex multi-tier sales and purchase taxes and charges:
//! - "On Net Total": tax calculated directly on invoice net total.
//! - "Actual": flat, explicit tax/charge amount.
//! - "On Previous Row Amount": tax calculated on the tax amount of a prior tax row.
//! - "On Previous Row Total": tax calculated on cumulative cumulative total through a prior tax row.
//! - Inclusive tax back-calculation (taxes included in print rate).
//! - Automatic round-off calculations.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of tax computation formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargeType {
    /// Percentage calculated on invoice net total.
    OnNetTotal,
    /// Absolute fixed monetary amount.
    Actual,
    /// Percentage applied to the tax amount of a previous tax row.
    OnPreviousRowAmount,
    /// Percentage applied to the cumulative invoice total of a previous tax row.
    OnPreviousRowTotal,
}

/// A single row in the Sales/Purchase Taxes and Charges table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxCharge {
    pub idx: usize,
    pub charge_type: ChargeType,
    pub account_head: CompactString,
    pub description: CompactString,
    pub rate: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub included_in_print_rate: bool,
    /// Target previous row index for `OnPreviousRowAmount` / `OnPreviousRowTotal` (1-based index).
    pub row_id: Option<usize>,
}

/// Result of complete taxes, totals, and round-off calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxCalculationResult {
    pub net_total: Decimal,
    pub total_taxes_and_charges: Decimal,
    pub grand_total: Decimal,
    pub rounding_adjustment: Decimal,
    pub rounded_total: Decimal,
    pub charges: Vec<TaxCharge>,
}

/// Engine executing relational taxes and cumulative total calculations.
pub struct TaxCalculationEngine;

impl TaxCalculationEngine {
    /// Calculates all tax charge lines and resulting grand total.
    pub fn calculate(
        net_total: Decimal,
        charges: &[TaxCharge],
        enable_round_off: bool,
    ) -> TaxCalculationResult {
        let mut computed_charges: Vec<TaxCharge> = charges.to_vec();
        let mut running_total = net_total;
        let mut total_taxes = Decimal::ZERO;

        for i in 0..computed_charges.len() {
            let charge_type = computed_charges[i].charge_type;
            let rate = computed_charges[i].rate;
            let rate_factor = rate / Decimal::from(100);

            let tax_amount = match charge_type {
                ChargeType::OnNetTotal => net_total * rate_factor,
                ChargeType::Actual => computed_charges[i].tax_amount,
                ChargeType::OnPreviousRowAmount => {
                    let prev_idx = computed_charges[i].row_id.unwrap_or(1).saturating_sub(1);
                    if prev_idx < i {
                        computed_charges[prev_idx].tax_amount * rate_factor
                    } else {
                        Decimal::ZERO
                    }
                }
                ChargeType::OnPreviousRowTotal => {
                    let prev_idx = computed_charges[i].row_id.unwrap_or(1).saturating_sub(1);
                    if prev_idx < i {
                        computed_charges[prev_idx].total * rate_factor
                    } else {
                        running_total * rate_factor
                    }
                }
            };

            computed_charges[i].tax_amount = tax_amount;

            // In Frappe, if included_in_print_rate, tax is part of base total, not added on top
            if !computed_charges[i].included_in_print_rate {
                running_total += tax_amount;
                total_taxes += tax_amount;
            } else {
                total_taxes += tax_amount;
            }

            computed_charges[i].total = running_total;
        }

        let grand_total = running_total;

        let (rounded_total, rounding_adjustment) = if enable_round_off {
            let rounded = grand_total.round();
            let adjustment = rounded - grand_total;
            (rounded, adjustment)
        } else {
            (grand_total, Decimal::ZERO)
        };

        TaxCalculationResult {
            net_total,
            total_taxes_and_charges: total_taxes,
            grand_total,
            rounding_adjustment,
            rounded_total,
            charges: computed_charges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_multi_tier_tax_calculation_with_previous_row_formula() {
        let net_total = dec!(1000.00);

        // Tier 1: 10% VAT on Net Total = $100 -> Total = $1100
        // Tier 2: 5% Cess on Previous Row Tax Amount = 5% of $100 = $5 -> Total = $1105
        // Tier 3: Flat Shipping Charge of $25 -> Total = $1130
        let charges = vec![
            TaxCharge {
                idx: 1,
                charge_type: ChargeType::OnNetTotal,
                account_head: "2200 - Output VAT".into(),
                description: "VAT 10%".into(),
                rate: dec!(10.0),
                tax_amount: Decimal::ZERO,
                total: Decimal::ZERO,
                included_in_print_rate: false,
                row_id: None,
            },
            TaxCharge {
                idx: 2,
                charge_type: ChargeType::OnPreviousRowAmount,
                account_head: "2210 - Education Cess".into(),
                description: "Cess 5% on VAT".into(),
                rate: dec!(5.0),
                tax_amount: Decimal::ZERO,
                total: Decimal::ZERO,
                included_in_print_rate: false,
                row_id: Some(1),
            },
            TaxCharge {
                idx: 3,
                charge_type: ChargeType::Actual,
                account_head: "5120 - Shipping Charges".into(),
                description: "Fixed Express Shipping".into(),
                rate: Decimal::ZERO,
                tax_amount: dec!(25.00),
                total: Decimal::ZERO,
                included_in_print_rate: false,
                row_id: None,
            },
        ];

        let result = TaxCalculationEngine::calculate(net_total, &charges, true);

        assert_eq!(result.net_total, dec!(1000.00));
        assert_eq!(result.charges[0].tax_amount, dec!(100.00));
        assert_eq!(result.charges[0].total, dec!(1100.00));

        assert_eq!(result.charges[1].tax_amount, dec!(5.00));
        assert_eq!(result.charges[1].total, dec!(1105.00));

        assert_eq!(result.charges[2].tax_amount, dec!(25.00));
        assert_eq!(result.charges[2].total, dec!(1130.00));

        assert_eq!(result.grand_total, dec!(1130.00));
        assert_eq!(result.total_taxes_and_charges, dec!(130.00));
        assert_eq!(result.rounded_total, dec!(1130.00));
    }
}
