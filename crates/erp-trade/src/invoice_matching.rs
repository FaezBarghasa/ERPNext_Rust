//! Computer Vision OCR & Automated 3-Way Invoice Matching (`erp-trade::invoice_matching`).
//!
//! Reconciles supplier purchase invoices against Goods Receipts (quantities) and Purchase Orders (unit rates).
//! $$\text{Matched} \iff \text{Invoice}(\text{Qty}, \text{Rate}) \equiv \text{Receipt}(\text{Qty}) \land \text{PurchaseOrder}(\text{Rate})$$
//! Auto-approves and posts draft payment entries when variances are within predefined tolerance ($\le 0.5\%$).

use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Purchase order item reference.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PoItemRef {
    pub item_code: CompactString,
    pub ordered_qty: Decimal,
    pub contract_rate: Decimal,
}

/// Goods receipt note (GRN) item line.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GrnItemLine {
    pub item_code: CompactString,
    pub received_qty: Decimal,
    pub accepted_qty: Decimal,
}

/// Ingested Vendor Invoice line from OCR.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OcrInvoiceLine {
    pub item_code: CompactString,
    pub billed_qty: Decimal,
    pub billed_rate: Decimal,
    pub line_total: Decimal,
}

/// Discrepancy analysis for an invoice line.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MatchVariance {
    pub item_code: CompactString,
    pub qty_variance: Decimal,
    pub rate_variance: Decimal,
    pub amount_variance: Decimal,
    pub variance_percentage: Decimal,
    pub is_within_tolerance: bool,
}

/// Outcome of the automated 3-way matching process.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThreeWayMatchResult {
    pub invoice_number: CompactString,
    pub is_fully_matched: bool,
    pub auto_post_payment: bool,
    pub total_billed_amount: Decimal,
    pub total_expected_amount: Decimal,
    pub variances: Vec<MatchVariance>,
    pub status: CompactString, // "Matched", "VarianceReview", "Rejected"
}

pub struct InvoiceMatchingEngine;

impl InvoiceMatchingEngine {
    /// Evaluates 3-way matching with a configurable tolerance percentage (default: 0.5%).
    #[must_use]
    pub fn evaluate_match(
        invoice_number: &str,
        invoice_lines: &[OcrInvoiceLine],
        po_items: &[PoItemRef],
        grn_items: &[GrnItemLine],
        tolerance_pct: Decimal,
    ) -> ThreeWayMatchResult {
        let mut variances = Vec::new();
        let mut is_fully_matched = true;
        let mut total_billed = Decimal::ZERO;
        let mut total_expected = Decimal::ZERO;

        for inv_line in invoice_lines {
            total_billed += inv_line.line_total;

            let po_item = po_items.iter().find(|p| p.item_code == inv_line.item_code);
            let grn_item = grn_items.iter().find(|g| g.item_code == inv_line.item_code);

            let expected_rate = po_item.map_or(Decimal::ZERO, |p| p.contract_rate);
            let accepted_qty = grn_item.map_or(Decimal::ZERO, |g| g.accepted_qty);
            let expected_line_total = accepted_qty * expected_rate;
            total_expected += expected_line_total;

            let qty_var = inv_line.billed_qty - accepted_qty;
            let rate_var = inv_line.billed_rate - expected_rate;
            let amt_var = (inv_line.line_total - expected_line_total).abs();

            let var_pct = if expected_line_total > Decimal::ZERO {
                (amt_var / expected_line_total) * dec!(100.0)
            } else if inv_line.line_total > Decimal::ZERO {
                dec!(100.0)
            } else {
                Decimal::ZERO
            };

            let is_ok = var_pct <= tolerance_pct;
            if !is_ok {
                is_fully_matched = false;
            }

            variances.push(MatchVariance {
                item_code: inv_line.item_code.clone(),
                qty_variance: qty_var,
                rate_variance: rate_var,
                amount_variance: amt_var,
                variance_percentage: var_pct,
                is_within_tolerance: is_ok,
            });
        }

        let status = if is_fully_matched {
            "Matched"
        } else if variances.iter().any(|v| v.variance_percentage > dec!(5.0)) {
            "Rejected"
        } else {
            "VarianceReview"
        };

        ThreeWayMatchResult {
            invoice_number: invoice_number.into(),
            is_fully_matched,
            auto_post_payment: is_fully_matched,
            total_billed_amount: total_billed,
            total_expected_amount: total_expected,
            variances,
            status: status.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_three_way_match() {
        let po = vec![PoItemRef {
            item_code: "STEEL-BAR-10MM".into(),
            ordered_qty: dec!(100),
            contract_rate: dec!(45.00),
        }];
        let grn = vec![GrnItemLine {
            item_code: "STEEL-BAR-10MM".into(),
            received_qty: dec!(100),
            accepted_qty: dec!(100),
        }];
        let inv = vec![OcrInvoiceLine {
            item_code: "STEEL-BAR-10MM".into(),
            billed_qty: dec!(100),
            billed_rate: dec!(45.00),
            line_total: dec!(4500.00),
        }];

        let result =
            InvoiceMatchingEngine::evaluate_match("INV-VEND-992", &inv, &po, &grn, dec!(0.5));
        assert!(result.is_fully_matched);
        assert!(result.auto_post_payment);
        assert_eq!(result.status.as_str(), "Matched");
        assert_eq!(result.total_billed_amount, dec!(4500.00));
    }

    #[test]
    fn test_variance_within_tolerance() {
        let po = vec![PoItemRef {
            item_code: "BOLT-M8".into(),
            ordered_qty: dec!(1000),
            contract_rate: dec!(1.00),
        }];
        let grn = vec![GrnItemLine {
            item_code: "BOLT-M8".into(),
            received_qty: dec!(1000),
            accepted_qty: dec!(1000),
        }];
        // Slight freight surcharge added: $1003.00 instead of $1000.00 (0.3% variance <= 0.5% tolerance)
        let inv = vec![OcrInvoiceLine {
            item_code: "BOLT-M8".into(),
            billed_qty: dec!(1000),
            billed_rate: dec!(1.003),
            line_total: dec!(1003.00),
        }];

        let result =
            InvoiceMatchingEngine::evaluate_match("INV-VEND-993", &inv, &po, &grn, dec!(0.5));
        assert!(result.is_fully_matched);
        assert!(result.auto_post_payment);
    }
}
