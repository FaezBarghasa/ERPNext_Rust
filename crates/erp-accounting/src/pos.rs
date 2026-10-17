//! Point-of-Sale (POS) Transaction Controller Suite (`erp-accounting::pos`).
//!
//! Provides upstream parity for:
//! - `POS Opening Entry` (Register shift start with opening cash float)
//! - `POS Invoice` (Fast retail sales, change calculation, multi-mode payments)
//! - `POS Closing Entry` (End of shift drawer reconciliation, over/short variances)
//! - Automatic consolidation into canonical `Sales Invoice` and balanced `GL Entry`.

use crate::ledger::{AccountingError, GlEntry};
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a POS shift register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum POSShiftStatus {
    Open,
    Closed,
}

/// A POS Opening Entry recording opening float cash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSOpeningEntry {
    pub name: CompactString,
    pub pos_profile: CompactString,
    pub user: CompactString,
    pub company: CompactString,
    pub period_start_date: DateTime<Utc>,
    pub opening_balance: Decimal,
    pub status: POSShiftStatus,
    pub docstatus: u8,
}

impl POSOpeningEntry {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        pos_profile: impl Into<CompactString>,
        user: impl Into<CompactString>,
        company: impl Into<CompactString>,
        opening_balance: Decimal,
    ) -> Self {
        Self {
            name: name.into(),
            pos_profile: pos_profile.into(),
            user: user.into(),
            company: company.into(),
            period_start_date: Utc::now(),
            opening_balance,
            status: POSShiftStatus::Open,
            docstatus: 1, // Submitted
        }
    }
}

/// Payment mode line on a POS Invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSInvoicePayment {
    pub mode_of_payment: CompactString,
    pub account: CompactString,
    pub amount: Decimal,
}

/// Item line on a POS Invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSInvoiceItem {
    pub item_code: CompactString,
    pub item_name: CompactString,
    pub qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub income_account: CompactString,
    pub expense_account: CompactString,
    pub cost_center: CompactString,
}

/// A submittable POS Invoice retail transaction document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSInvoice {
    pub name: CompactString,
    pub pos_profile: CompactString,
    pub pos_opening_entry: CompactString,
    pub customer: CompactString,
    pub company: CompactString,
    pub posting_date: DateTime<Utc>,
    pub items: Vec<POSInvoiceItem>,
    pub payments: Vec<POSInvoicePayment>,
    pub net_total: Decimal,
    pub grand_total: Decimal,
    pub paid_amount: Decimal,
    pub change_amount: Decimal,
    pub docstatus: u8, // 0: Draft, 1: Submitted, 2: Cancelled
}

impl POSInvoice {
    /// Validates invoice calculations, payments, and change amount.
    pub fn validate(&self) -> Result<(), AccountingError> {
        let calculated_total: Decimal = self.items.iter().map(|i| i.amount).sum();
        if calculated_total != self.grand_total {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit: calculated_total,
                total_credit: self.grand_total,
            });
        }

        let total_paid: Decimal = self.payments.iter().map(|p| p.amount).sum();
        if total_paid < self.grand_total {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit: total_paid,
                total_credit: self.grand_total,
            });
        }

        Ok(())
    }

    /// Generates balanced General Ledger entries for the retail invoice.
    pub fn generate_gl_entries(
        &self,
        _receivable_account: &str,
    ) -> Result<Vec<GlEntry>, AccountingError> {
        self.validate()?;
        let mut entries = Vec::new();
        let posting = self.posting_date.date_naive();

        // 1. Credit Income accounts
        for item in &self.items {
            entries.push(GlEntry {
                posting_date: posting,
                account: item.income_account.to_string(),
                debit: Decimal::ZERO,
                credit: item.amount,
                voucher_type: "POS Invoice".into(),
                voucher_no: self.name.to_string(),
                remarks: format!("Sales: {} x {}", item.item_code, item.qty),
            });
        }

        // 2. Debit Payments / Cash / Bank accounts
        for payment in &self.payments {
            // Deduct change from primary cash mode if applicable
            let net_payment = if payment.mode_of_payment == "Cash" {
                payment.amount - self.change_amount
            } else {
                payment.amount
            };

            entries.push(GlEntry {
                posting_date: posting,
                account: payment.account.to_string(),
                debit: net_payment,
                credit: Decimal::ZERO,
                voucher_type: "POS Invoice".into(),
                voucher_no: self.name.to_string(),
                remarks: format!("Payment via {}", payment.mode_of_payment),
            });
        }

        // Verify zero-drift
        let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();
        if total_debit != total_credit {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit,
                total_credit,
            });
        }

        Ok(entries)
    }
}

/// A line in the POS Closing Entry reconciling expected vs counted amounts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSClosingPaymentReconciliation {
    pub mode_of_payment: CompactString,
    pub opening_amount: Decimal,
    pub expected_amount: Decimal,
    pub closing_amount: Decimal,
    pub difference: Decimal,
}

/// POS Closing Entry recording end of shift cash reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct POSClosingEntry {
    pub name: CompactString,
    pub pos_profile: CompactString,
    pub pos_opening_entry: CompactString,
    pub user: CompactString,
    pub company: CompactString,
    pub period_end_date: DateTime<Utc>,
    pub payment_reconciliations: Vec<POSClosingPaymentReconciliation>,
    pub total_collected: Decimal,
    pub total_variance: Decimal,
    pub docstatus: u8,
}

impl POSClosingEntry {
    /// Consolidates a set of POS invoices into a closing shift summary.
    #[must_use]
    pub fn consolidate_shift(
        name: impl Into<CompactString>,
        opening: &POSOpeningEntry,
        invoices: &[POSInvoice],
        closing_cash_counted: Decimal,
    ) -> Self {
        let mut expected_cash = opening.opening_balance;
        let mut total_collected = Decimal::ZERO;

        for inv in invoices {
            if inv.docstatus == 1 {
                for pay in &inv.payments {
                    total_collected += pay.amount;
                    if pay.mode_of_payment == "Cash" {
                        expected_cash += pay.amount - inv.change_amount;
                    }
                }
            }
        }

        let cash_diff = closing_cash_counted - expected_cash;

        let reconciliations = vec![POSClosingPaymentReconciliation {
            mode_of_payment: "Cash".into(),
            opening_amount: opening.opening_balance,
            expected_amount: expected_cash,
            closing_amount: closing_cash_counted,
            difference: cash_diff,
        }];

        Self {
            name: name.into(),
            pos_profile: opening.pos_profile.clone(),
            pos_opening_entry: opening.name.clone(),
            user: opening.user.clone(),
            company: opening.company.clone(),
            period_end_date: Utc::now(),
            payment_reconciliations: reconciliations,
            total_collected,
            total_variance: cash_diff,
            docstatus: 1, // Submitted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pos_shift_lifecycle_and_gl() {
        let opening = POSOpeningEntry::new(
            "POS-OPEN-2026-0001",
            "Main Retail",
            "cashier@example.com",
            "Acme Retail",
            dec!(200.00),
        );
        assert_eq!(opening.opening_balance, dec!(200.00));

        let invoice = POSInvoice {
            name: "POS-INV-2026-0001".into(),
            pos_profile: "Main Retail".into(),
            pos_opening_entry: opening.name.clone(),
            customer: "Walk-in Customer".into(),
            company: "Acme Retail".into(),
            posting_date: Utc::now(),
            items: vec![
                POSInvoiceItem {
                    item_code: "COFFEE-MUG".into(),
                    item_name: "Coffee Mug".into(),
                    qty: dec!(2),
                    rate: dec!(15.00),
                    amount: dec!(30.00),
                    income_account: "Sales - AR".into(),
                    expense_account: "COGS - AR".into(),
                    cost_center: "Main - AR".into(),
                },
                POSInvoiceItem {
                    item_code: "COFFEE-BEANS".into(),
                    item_name: "Arabica Beans 500g".into(),
                    qty: dec!(1),
                    rate: dec!(20.00),
                    amount: dec!(20.00),
                    income_account: "Sales - AR".into(),
                    expense_account: "COGS - AR".into(),
                    cost_center: "Main - AR".into(),
                },
            ],
            payments: vec![POSInvoicePayment {
                mode_of_payment: "Cash".into(),
                account: "Cash Drawer - AR".into(),
                amount: dec!(60.00),
            }],
            net_total: dec!(50.00),
            grand_total: dec!(50.00),
            paid_amount: dec!(60.00),
            change_amount: dec!(10.00),
            docstatus: 1,
        };

        let gl_entries = invoice.generate_gl_entries("Debtors - AR").unwrap();
        assert_eq!(gl_entries.len(), 3);

        let total_debit: Decimal = gl_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = gl_entries.iter().map(|e| e.credit).sum();
        assert_eq!(total_debit, dec!(50.00));
        assert_eq!(total_credit, dec!(50.00));

        // Consolidate closing shift
        let closing = POSClosingEntry::consolidate_shift(
            "POS-CLOSE-2026-0001",
            &opening,
            &[invoice],
            dec!(250.00), // Exactly opening 200 + net cash 50
        );

        assert_eq!(closing.payment_reconciliations[0].difference, dec!(0.00));
        assert_eq!(closing.total_variance, dec!(0.00));
    }
}
