//! Relational Tax Withholding & Tax-Inclusive Pricing Engine (`erp-accounting::tax_withholding`).
//!
//! Provides:
//! - Relational `Tax Withholding Entry` child records
//! - Relational `Item Wise Tax Detail` child records
//! - Pre-rounding tax-inclusive pricing calculation:
//!   $$\text{Base Amount} = \frac{\text{Inclusive Amount}}{1 + \sum \text{Tax Rates}}$$
//!   $$\text{Tax Amount} = \text{Inclusive Amount} - \text{Base Amount}$$

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Relational Tax Withholding (TDS/TCS) Entry child table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaxWithholdingEntry {
    pub tax_withholding_category: CompactString,
    pub certificate_number: Option<CompactString>,
    pub pan_or_tax_id: CompactString,
    pub taxable_amount: Decimal,
    pub tax_rate_pct: Decimal,
    pub tax_amount: Decimal,
    pub fiscal_year: CompactString,
}

/// Relational Item-Wise Tax Detail child record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemWiseTaxDetail {
    pub item_code: CompactString,
    pub tax_type: CompactString, // e.g. "VAT 19%", "CGST 9%", "SGST 9%"
    pub tax_rate_pct: Decimal,
    pub taxable_amount: Decimal,
    pub tax_amount: Decimal,
}

/// Tax calculation engine for tax-inclusive and tax-exclusive pricing.
pub struct TaxPricingEngine;

impl TaxPricingEngine {
    /// Computes base amount and tax amounts for a tax-inclusive total price
    /// using pre-rounding formula to prevent integer rounding mismatches.
    #[must_use]
    pub fn calculate_tax_inclusive_split(
        inclusive_amount: Decimal,
        tax_rates_pct: &[Decimal],
    ) -> (Decimal, Vec<Decimal>) {
        let sum_rates: Decimal =
            tax_rates_pct.iter().copied().sum::<Decimal>() / Decimal::from(100);
        let divisor = Decimal::ONE + sum_rates;

        let base_amount = if divisor.is_zero() {
            inclusive_amount
        } else {
            inclusive_amount / divisor
        };

        let mut taxes = Vec::with_capacity(tax_rates_pct.len());
        for rate in tax_rates_pct {
            let tax_amt = base_amount * (*rate / Decimal::from(100));
            taxes.push(tax_amt);
        }

        (base_amount, taxes)
    }

    /// Computes TDS withholding for a transaction if net party balance is positive.
    #[must_use]
    pub fn calculate_tds(
        taxable_amount: Decimal,
        rate_pct: Decimal,
        threshold: Decimal,
        cumulative_party_total: Decimal,
    ) -> Option<Decimal> {
        let total_with_current = cumulative_party_total + taxable_amount;
        if total_with_current > threshold && taxable_amount > Decimal::ZERO {
            Some(taxable_amount * (rate_pct / Decimal::from(100)))
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
    fn test_tax_inclusive_split_calculation() {
        let inclusive = dec!(119.00);
        let rates = vec![dec!(19.0)]; // 19% VAT

        let (base, taxes) = TaxPricingEngine::calculate_tax_inclusive_split(inclusive, &rates);
        assert_eq!(base, dec!(100.00));
        assert_eq!(taxes[0], dec!(19.00));
    }

    #[test]
    fn test_tds_threshold_enforcement() {
        let rate = dec!(10.0);
        let threshold = dec!(50000.0);

        // Under threshold: zero TDS
        let tds1 = TaxPricingEngine::calculate_tds(dec!(20000.0), rate, threshold, dec!(10000.0));
        assert_eq!(tds1, None);

        // Above threshold: applies TDS
        let tds2 = TaxPricingEngine::calculate_tds(dec!(30000.0), rate, threshold, dec!(35000.0));
        assert_eq!(tds2, Some(dec!(3000.0)));
    }
}
