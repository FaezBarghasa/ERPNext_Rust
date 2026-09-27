use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Calculation method for a tax row in a multi-tier tax schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxType {
    /// Applied directly to net line total: $\text{Tax} = \text{Net} \times \frac{\text{Rate}}{100}$
    OnNetTotal,
    /// Compounded on net plus specific previous tax line rows:
    /// $\text{Tax}_k = (\text{Net} + \sum_{j < k} \text{Tax}_j) \times \frac{\text{Rate}}{100}$
    CompoundedOnPrevious,
    /// Back-calculated from tax-inclusive net line total.
    InclusiveInNet,
}

/// Tax schedule row template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxRow {
    /// Account head to book tax to (e.g. "2210 - Output VAT").
    pub account_head: String,
    /// Rate in percentage (e.g. 10.0 for 10%).
    pub rate: Decimal,
    /// Tax calculation method.
    pub charge_type: TaxType,
}

/// Individual computed tax line output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxLineResult {
    /// Target tax account.
    pub account_head: String,
    /// Tax rate.
    pub rate: Decimal,
    /// Calculated tax amount.
    pub tax_amount: Decimal,
    /// Cumulative total including this tax line.
    pub total: Decimal,
}

/// Result of complete multi-tier tax calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxScheduleResult {
    /// Net taxable base total.
    pub net_total: Decimal,
    /// Total calculated taxes across all rows.
    pub total_tax: Decimal,
    /// Grand total (Net + Taxes).
    pub grand_total: Decimal,
    /// Detailed computed tax lines.
    pub tax_lines: Vec<TaxLineResult>,
}

/// Compounding Multi-Tier Tax Engine (Milestone 2.10).
pub struct TaxEngine;

impl TaxEngine {
    /// Computes compounding multi-tier taxes across tax schedule rows.
    #[must_use]
    pub fn calculate_taxes(base_net_amount: Decimal, tax_rows: &[TaxRow]) -> TaxScheduleResult {
        let mut running_tax_total = Decimal::ZERO;
        let mut tax_lines = Vec::new();

        for row in tax_rows {
            let row_tax = match row.charge_type {
                TaxType::OnNetTotal => base_net_amount * (row.rate / Decimal::from(100)),
                TaxType::CompoundedOnPrevious => {
                    (base_net_amount + running_tax_total) * (row.rate / Decimal::from(100))
                }
                TaxType::InclusiveInNet => {
                    // Tax = Base - (Base / (1 + Rate/100))
                    let factor = Decimal::ONE + (row.rate / Decimal::from(100));
                    base_net_amount - (base_net_amount / factor)
                }
            };

            running_tax_total += row_tax;

            tax_lines.push(TaxLineResult {
                account_head: row.account_head.clone(),
                rate: row.rate,
                tax_amount: row_tax,
                total: base_net_amount + running_tax_total,
            });
        }

        TaxScheduleResult {
            net_total: base_net_amount,
            total_tax: running_tax_total,
            grand_total: base_net_amount + running_tax_total,
            tax_lines,
        }
    }
}
