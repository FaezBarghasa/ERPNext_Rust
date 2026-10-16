//! Transactional Purchase Invoice Controller (`erp-accounting::purchase_invoice`).
//!
//! Implements canonical upstream ERPNext Purchase Invoice business logic:
//! - Submittable accounts payable document (Draft -> Submitted -> Cancelled)
//! - Line items with unit rates, discounts, expense accounts, and cost centers
//! - Taxes & Charges calculation (VAT/GST input tax credits)
//! - 3-Way Matching validation vs Purchase Order (rate) and Purchase Receipt (qty)
//! - Automated General Ledger double-entry generation (Expense/Interim Debit, Tax Debit, Creditor Credit)
//! - Payment Ledger Entry (PLE) generation for Accounts Payable (AP) tracking.

use crate::ledger::{AccountingError, JournalEntryLine};
use crate::payment_ledger::PaymentLedgerEntry;
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Purchase invoice domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PurchaseInvoiceError {
    #[error("Purchase Invoice must contain at least one line item")]
    EmptyLineItems,
    #[error("Invalid line item rate or qty for item '{item_code}': qty={qty}, rate={rate}")]
    InvalidLineItem {
        item_code: CompactString,
        qty: Decimal,
        rate: Decimal,
    },
    #[error("Exchange rate must be positive, got {0}")]
    InvalidExchangeRate(Decimal),
    #[error("3-Way Match validation failed for item '{item_code}': {reason}")]
    ThreeWayMatchFailed {
        item_code: CompactString,
        reason: String,
    },
    #[error("Cannot submit document with status {0}, expected Draft (0)")]
    InvalidDocStatus(i32),
    #[error("Accounting ledger error: {0}")]
    Accounting(#[from] AccountingError),
}

/// An individual line item on a Purchase Invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurchaseInvoiceItem {
    pub item_code: CompactString,
    pub item_name: CompactString,
    pub qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal, // qty * rate
    pub expense_account: CompactString,
    pub cost_center: Option<CompactString>,
    pub purchase_order: Option<CompactString>,
    pub purchase_receipt: Option<CompactString>,
    pub po_contract_rate: Option<Decimal>,
    pub pr_accepted_qty: Option<Decimal>,
}

/// A tax or charge line item (VAT/GST/Customs Duty).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurchaseTaxItem {
    pub charge_type: CompactString,  // "On Net Total", "Actual"
    pub account_head: CompactString, // e.g. "Input VAT 10% - Enterprise"
    pub rate: Decimal,               // e.g. 10.0 for 10%
    pub tax_amount: Decimal,
    pub total: Decimal,
}

/// Canonical ERPNext Purchase Invoice entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurchaseInvoice {
    pub name: CompactString,
    pub supplier: CompactString,
    pub posting_date: NaiveDate,
    pub due_date: NaiveDate,
    pub company: CompactString,
    pub credit_to: CompactString, // Creditors / Accounts Payable account
    pub currency: CompactString,
    pub conversion_rate: Decimal,
    pub items: Vec<PurchaseInvoiceItem>,
    pub taxes: Vec<PurchaseTaxItem>,
    pub net_total: Decimal,
    pub total_taxes_and_charges: Decimal,
    pub grand_total: Decimal,
    pub docstatus: i32, // 0 = Draft, 1 = Submitted, 2 = Cancelled
    pub remarks: Option<String>,
}

impl PurchaseInvoice {
    /// Validates line items, recalculates totals, and executes 3-way matching rules.
    pub fn validate(&mut self, tolerance_percent: Decimal) -> Result<(), PurchaseInvoiceError> {
        if self.items.is_empty() {
            return Err(PurchaseInvoiceError::EmptyLineItems);
        }
        if self.conversion_rate <= Decimal::ZERO {
            return Err(PurchaseInvoiceError::InvalidExchangeRate(
                self.conversion_rate,
            ));
        }

        let mut calculated_net = Decimal::ZERO;

        for item in &mut self.items {
            if item.qty <= Decimal::ZERO || item.rate < Decimal::ZERO {
                return Err(PurchaseInvoiceError::InvalidLineItem {
                    item_code: item.item_code.clone(),
                    qty: item.qty,
                    rate: item.rate,
                });
            }

            item.amount = item.qty * item.rate;
            calculated_net += item.amount;

            // 3-Way Match Verification vs PO contract rate
            if let Some(po_rate) = item.po_contract_rate
                && po_rate > Decimal::ZERO
            {
                let rate_diff = (item.rate - po_rate).abs();
                let diff_pct = (rate_diff / po_rate) * Decimal::from(100);
                if diff_pct > tolerance_percent {
                    return Err(PurchaseInvoiceError::ThreeWayMatchFailed {
                        item_code: item.item_code.clone(),
                        reason: format!(
                            "Billed rate ({}) exceeds PO contract rate ({}) by {:.2}% (tolerance is {}%)",
                            item.rate, po_rate, diff_pct, tolerance_percent
                        ),
                    });
                }
            }

            // 3-Way Match Verification vs PR accepted quantity
            if let Some(pr_qty) = item.pr_accepted_qty
                && item.qty > pr_qty
            {
                return Err(PurchaseInvoiceError::ThreeWayMatchFailed {
                    item_code: item.item_code.clone(),
                    reason: format!(
                        "Billed qty ({}) exceeds Goods Receipt accepted qty ({})",
                        item.qty, pr_qty
                    ),
                });
            }
        }

        self.net_total = calculated_net;

        // Recalculate taxes
        let mut total_taxes = Decimal::ZERO;
        let mut current_total = self.net_total;

        for tax in &mut self.taxes {
            if tax.charge_type == "On Net Total" {
                tax.tax_amount = (self.net_total * tax.rate) / Decimal::from(100);
            }
            current_total += tax.tax_amount;
            tax.total = current_total;
            total_taxes += tax.tax_amount;
        }

        self.total_taxes_and_charges = total_taxes;
        self.grand_total = self.net_total + self.total_taxes_and_charges;

        Ok(())
    }

    /// Submits the Purchase Invoice, validating invariants and transitioning docstatus to 1.
    pub fn submit(&mut self, tolerance_percent: Decimal) -> Result<(), PurchaseInvoiceError> {
        if self.docstatus != 0 {
            return Err(PurchaseInvoiceError::InvalidDocStatus(self.docstatus));
        }
        self.validate(tolerance_percent)?;
        self.docstatus = 1;
        Ok(())
    }

    /// Generates balanced double-entry General Ledger journal lines.
    ///
    /// Equation:
    /// $$\sum \text{Debit (Expense + Tax)} = \sum \text{Credit (Accounts Payable)}$$
    pub fn generate_gl_entries(&self) -> Result<Vec<JournalEntryLine>, PurchaseInvoiceError> {
        let mut lines = Vec::new();

        // 1. Debit Expense / Stock Clearing accounts for each item
        for item in &self.items {
            let base_amount = item.amount * self.conversion_rate;
            lines.push(JournalEntryLine {
                account: item.expense_account.to_string(),
                debit: base_amount,
                credit: Decimal::ZERO,
                debit_in_account_currency: item.amount,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: self.conversion_rate,
                party_type: None,
                party: None,
            });
        }

        // 2. Debit Input Tax accounts
        for tax in &self.taxes {
            let base_tax = tax.tax_amount * self.conversion_rate;
            lines.push(JournalEntryLine {
                account: tax.account_head.to_string(),
                debit: base_tax,
                credit: Decimal::ZERO,
                debit_in_account_currency: tax.tax_amount,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: self.conversion_rate,
                party_type: None,
                party: None,
            });
        }

        // 3. Credit Accounts Payable / Creditor account
        let base_grand_total = self.grand_total * self.conversion_rate;
        lines.push(JournalEntryLine {
            account: self.credit_to.to_string(),
            debit: Decimal::ZERO,
            credit: base_grand_total,
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: self.grand_total,
            exchange_rate: self.conversion_rate,
            party_type: Some("Supplier".into()),
            party: Some(self.supplier.to_string()),
        });

        // Validate that transaction debits equal credits
        let total_debit: Decimal = lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = lines.iter().map(|l| l.credit).sum();

        if total_debit != total_credit {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit,
                total_credit,
            }
            .into());
        }

        Ok(lines)
    }

    /// Generates Payment Ledger Entry (PLE) row establishing the Accounts Payable liability.
    #[must_use]
    pub fn generate_ple_entry(&self) -> PaymentLedgerEntry {
        PaymentLedgerEntry {
            name: format!("{}-AP", self.name).into(),
            company: self.company.clone(),
            posting_date: self.posting_date,
            account_type: "Payable".into(),
            party_type: "Supplier".into(),
            party: self.supplier.clone(),
            account: self.credit_to.clone(),
            voucher_type: "Purchase Invoice".into(),
            voucher_no: self.name.clone(),
            against_voucher_type: None,
            against_voucher_no: None,
            amount: self.grand_total * self.conversion_rate,
            amount_in_account_currency: self.grand_total,
            currency: self.currency.clone(),
            delinked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payment_ledger::PaymentLedgerEngine;
    use rust_decimal_macros::dec;

    #[test]
    fn test_purchase_invoice_submission_3way_matching_and_gl_balance() {
        let mut pi = PurchaseInvoice {
            name: "ACC-PINV-2026-0001".into(),
            supplier: "ACME Industrial Supplies".into(),
            posting_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            company: "Enterprise Corp".into(),
            credit_to: "Creditors - Enterprise".into(),
            currency: "USD".into(),
            conversion_rate: dec!(1.0),
            items: vec![
                PurchaseInvoiceItem {
                    item_code: "STEEL-SHEET-01".into(),
                    item_name: "Hot Rolled Steel Sheet 2mm".into(),
                    qty: dec!(50),
                    rate: dec!(100),
                    amount: dec!(5000),
                    expense_account: "Stock Received But Not Billed - Enterprise".into(),
                    cost_center: Some("Main - EC".into()),
                    purchase_order: Some("PO-2026-001".into()),
                    purchase_receipt: Some("PR-2026-001".into()),
                    po_contract_rate: Some(dec!(100)), // Matched exactly
                    pr_accepted_qty: Some(dec!(50)),   // Matched exactly
                },
                PurchaseInvoiceItem {
                    item_code: "COPPER-WIRE-01".into(),
                    item_name: "Industrial Copper Wire 100m".into(),
                    qty: dec!(20),
                    rate: dec!(50),
                    amount: dec!(1000),
                    expense_account: "Raw Material Consumed - Enterprise".into(),
                    cost_center: Some("Main - EC".into()),
                    purchase_order: Some("PO-2026-002".into()),
                    purchase_receipt: Some("PR-2026-002".into()),
                    po_contract_rate: Some(dec!(50)),
                    pr_accepted_qty: Some(dec!(20)),
                },
            ],
            taxes: vec![PurchaseTaxItem {
                charge_type: "On Net Total".into(),
                account_head: "Input VAT 10% - Enterprise".into(),
                rate: dec!(10.0),
                tax_amount: dec!(600),
                total: dec!(6600),
            }],
            net_total: dec!(6000),
            total_taxes_and_charges: dec!(600),
            grand_total: dec!(6600),
            docstatus: 0,
            remarks: Some("Raw material procurement batch A".into()),
        };

        // Submit with 1.0% tolerance
        pi.submit(dec!(1.0)).expect("Submission must succeed");
        assert_eq!(pi.docstatus, 1);
        assert_eq!(pi.grand_total, dec!(6600));

        let gl_entries = pi.generate_gl_entries().expect("GL entries must balance");
        let total_debit: Decimal = gl_entries.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = gl_entries.iter().map(|l| l.credit).sum();

        assert_eq!(total_debit, total_credit);
        assert_eq!(total_debit, dec!(6600));

        // Verify Accounts Payable in PaymentLedgerEngine
        let mut ple_engine = PaymentLedgerEngine::new();
        ple_engine.record_entry(pi.generate_ple_entry());

        let ap_outstanding =
            ple_engine.calculate_outstanding("Purchase Invoice", "ACC-PINV-2026-0001");
        assert_eq!(ap_outstanding, dec!(6600));
    }

    #[test]
    fn test_purchase_invoice_3way_matching_rate_tolerance_violation() {
        let mut pi = PurchaseInvoice {
            name: "ACC-PINV-2026-0002".into(),
            supplier: "ACME Supplies".into(),
            posting_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            company: "Enterprise Corp".into(),
            credit_to: "Creditors - Enterprise".into(),
            currency: "USD".into(),
            conversion_rate: dec!(1.0),
            items: vec![PurchaseInvoiceItem {
                item_code: "STEEL-SHEET-01".into(),
                item_name: "Hot Rolled Steel Sheet 2mm".into(),
                qty: dec!(50),
                rate: dec!(120), // 20% higher than contract rate 100!
                amount: dec!(6000),
                expense_account: "Stock Received - Enterprise".into(),
                cost_center: None,
                purchase_order: Some("PO-2026-001".into()),
                purchase_receipt: None,
                po_contract_rate: Some(dec!(100)),
                pr_accepted_qty: None,
            }],
            taxes: vec![],
            net_total: dec!(6000),
            total_taxes_and_charges: dec!(0),
            grand_total: dec!(6000),
            docstatus: 0,
            remarks: None,
        };

        // Submit with 2.0% tolerance should reject the 20% price inflation
        let result = pi.submit(dec!(2.0));
        assert!(matches!(
            result,
            Err(PurchaseInvoiceError::ThreeWayMatchFailed { .. })
        ));
    }
}
