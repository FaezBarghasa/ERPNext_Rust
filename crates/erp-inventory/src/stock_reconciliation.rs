//! Transactional Stock Reconciliation Controller (`erp-inventory::stock_reconciliation`).
//!
//! Implements canonical upstream ERPNext Stock Reconciliation business logic:
//! - Physical stock audit adjustments and periodic inventory counts
//! - Resolves current warehouse stock bin levels (quantity & valuation rate)
//! - Computes exact quantity and monetary value differences per item
//! - Generates balanced Stock Ledger Entries (SLEs) to align system bins with physical reality
//! - Generates zero-drift General Ledger (GL) double-entry accounting records (Stock In Hand vs Stock Adjustment).

use crate::fifo::StockLedgerEntry;
use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::GlEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for Stock Reconciliation transactions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StockReconciliationError {
    #[error("Stock Reconciliation must contain at least one item line")]
    EmptyItems,
    #[error("Cannot submit document with status {0}, expected Draft (0)")]
    InvalidDocStatus(i32),
    #[error(
        "Negative inventory count prohibited for item '{item_code}' in warehouse '{warehouse}': count={qty}"
    )]
    NegativeCountProhibited {
        item_code: CompactString,
        warehouse: CompactString,
        qty: Decimal,
    },
    #[error("Negative valuation rate prohibited for item '{item_code}': rate={rate}")]
    NegativeRateProhibited {
        item_code: CompactString,
        rate: Decimal,
    },
    #[error("Warehouse account not found for warehouse '{0}'")]
    WarehouseAccountNotFound(CompactString),
    #[error(
        "Unbalanced GL double-entries: total debit {total_debit} != total credit {total_credit}"
    )]
    UnbalancedGlEntries {
        total_debit: Decimal,
        total_credit: Decimal,
    },
}

/// An individual line item in a Stock Reconciliation voucher.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockReconciliationItem {
    pub item_code: CompactString,
    pub warehouse: CompactString,
    /// Physical count / reconciled quantity.
    pub qty: Decimal,
    /// Reconciled unit valuation rate.
    pub valuation_rate: Decimal,
    /// Current system on-hand quantity before reconciliation.
    pub current_qty: Decimal,
    /// Current system valuation rate before reconciliation.
    pub current_valuation_rate: Decimal,
    /// Calculated difference in quantity (qty - current_qty).
    pub quantity_difference: Decimal,
    /// Calculated difference in monetary value.
    pub amount_difference: Decimal,
    pub serial_nos: Vec<CompactString>,
    pub batch_no: Option<CompactString>,
}

/// Canonical ERPNext Stock Reconciliation document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockReconciliation {
    pub name: CompactString,
    pub posting_date: NaiveDate,
    pub company: CompactString,
    pub purpose: CompactString, // "Stock Reconciliation", "Opening Stock"
    pub expense_account: CompactString, // Default "5110 - Stock Adjustment"
    pub cost_center: Option<CompactString>,
    pub items: Vec<StockReconciliationItem>,
    pub total_amount_difference: Decimal,
    pub docstatus: i32, // 0 = Draft, 1 = Submitted, 2 = Cancelled
    pub remarks: Option<String>,
}

impl StockReconciliation {
    /// Validates count rules, non-negativity invariants, and computes system differences.
    pub fn validate<F>(&mut self, mut get_current_stock: F) -> Result<(), StockReconciliationError>
    where
        F: FnMut(&str, &str) -> (Decimal, Decimal), // (item_code, warehouse) -> (current_qty, current_valuation_rate)
    {
        if self.items.is_empty() {
            return Err(StockReconciliationError::EmptyItems);
        }

        let mut total_diff = Decimal::ZERO;

        for item in &mut self.items {
            if item.qty < Decimal::ZERO {
                return Err(StockReconciliationError::NegativeCountProhibited {
                    item_code: item.item_code.clone(),
                    warehouse: item.warehouse.clone(),
                    qty: item.qty,
                });
            }
            if item.valuation_rate < Decimal::ZERO {
                return Err(StockReconciliationError::NegativeRateProhibited {
                    item_code: item.item_code.clone(),
                    rate: item.valuation_rate,
                });
            }

            // Fetch current system state if not already populated
            let (sys_qty, sys_rate) =
                get_current_stock(item.item_code.as_str(), item.warehouse.as_str());
            item.current_qty = sys_qty;
            item.current_valuation_rate = sys_rate;

            item.quantity_difference = item.qty - item.current_qty;

            let new_amount = item.qty * item.valuation_rate;
            let current_amount = item.current_qty * item.current_valuation_rate;
            item.amount_difference = new_amount - current_amount;

            total_diff += item.amount_difference;
        }

        self.total_amount_difference = total_diff;
        Ok(())
    }

    /// Submits the Stock Reconciliation document (docstatus 0 -> 1).
    pub fn submit<F>(&mut self, get_current_stock: F) -> Result<(), StockReconciliationError>
    where
        F: FnMut(&str, &str) -> (Decimal, Decimal),
    {
        if self.docstatus != 0 {
            return Err(StockReconciliationError::InvalidDocStatus(self.docstatus));
        }
        self.validate(get_current_stock)?;
        self.docstatus = 1;
        Ok(())
    }

    /// Generates adjusting Stock Ledger Entries (SLEs).
    pub fn generate_stock_ledger_entries(&self) -> Vec<StockLedgerEntry> {
        let mut sles = Vec::new();
        let voucher_type = "Stock Reconciliation".to_string();
        let voucher_no = self.name.to_string();

        for (idx, item) in self.items.iter().enumerate() {
            // Only generate an SLE if there is a quantity or valuation difference
            if item.quantity_difference != Decimal::ZERO || item.amount_difference != Decimal::ZERO
            {
                sles.push(StockLedgerEntry {
                    name: format!("{}-sle-{}", self.name, idx + 1),
                    item_code: item.item_code.to_string(),
                    warehouse: item.warehouse.to_string(),
                    actual_qty: item.quantity_difference,
                    incoming_rate: item.valuation_rate,
                    valuation_rate: item.valuation_rate,
                    stock_value_difference: item.amount_difference,
                    posting_date: self.posting_date,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                });
            }
        }

        sles
    }

    /// Generates strictly balanced General Ledger (GL) entries for the net inventory difference.
    ///
    /// Accounting logic:
    /// - For positive amount difference (Inventory Gain):
    ///   - Debit: Warehouse Stock In Hand account (Asset increases)
    ///   - Credit: Stock Adjustment account (Income / Cost offset)
    /// - For negative amount difference (Inventory Loss/Shrinkage):
    ///   - Debit: Stock Adjustment account (Expense / Loss recognized)
    ///   - Credit: Warehouse Stock In Hand account (Asset decreases)
    pub fn generate_gl_entries<F>(
        &self,
        mut get_warehouse_account: F,
    ) -> Result<Vec<GlEntry>, StockReconciliationError>
    where
        F: FnMut(&str) -> Result<CompactString, StockReconciliationError>,
    {
        let mut entries = Vec::new();
        let voucher_type = "Stock Reconciliation".to_string();
        let voucher_no = self.name.to_string();
        let company = self.company.to_string();

        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for (idx, item) in self.items.iter().enumerate() {
            if item.amount_difference == Decimal::ZERO {
                continue;
            }

            let stock_account = get_warehouse_account(item.warehouse.as_str())?;

            if item.amount_difference > Decimal::ZERO {
                let diff = item.amount_difference;

                // Debit Stock In Hand
                entries.push(GlEntry {
                    name: format!("{}-gl-in-{}", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: stock_account.to_string(),
                    debit: diff,
                    credit: Decimal::ZERO,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                    party_type: None,
                    party: None,
                    company: company.clone(),
                });
                total_debit += diff;

                // Credit Stock Adjustment
                entries.push(GlEntry {
                    name: format!("{}-gl-adj-{}", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: self.expense_account.to_string(),
                    debit: Decimal::ZERO,
                    credit: diff,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                    party_type: None,
                    party: None,
                    company: company.clone(),
                });
                total_credit += diff;
            } else {
                let loss = -item.amount_difference;

                // Debit Stock Adjustment (Loss/Wastage)
                entries.push(GlEntry {
                    name: format!("{}-gl-adj-{}", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: self.expense_account.to_string(),
                    debit: loss,
                    credit: Decimal::ZERO,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                    party_type: None,
                    party: None,
                    company: company.clone(),
                });
                total_debit += loss;

                // Credit Stock In Hand
                entries.push(GlEntry {
                    name: format!("{}-gl-out-{}", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: stock_account.to_string(),
                    debit: Decimal::ZERO,
                    credit: loss,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                    party_type: None,
                    party: None,
                    company: company.clone(),
                });
                total_credit += loss;
            }
        }

        if total_debit != total_credit {
            return Err(StockReconciliationError::UnbalancedGlEntries {
                total_debit,
                total_credit,
            });
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_stock_reconciliation_gain_and_loss_gl_balance() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        let mut rec = StockReconciliation {
            name: "REC-2026-001".into(),
            posting_date: date,
            company: "Acme Industrial".into(),
            purpose: "Stock Reconciliation".into(),
            expense_account: "5110 - Stock Adjustment".into(),
            cost_center: Some("Main Factory".into()),
            items: vec![
                // Item 1: Physical count found 120 (system had 100 @ $10 = $1,000). Reconciled @ $10 = $1,200. +$200 Gain.
                StockReconciliationItem {
                    item_code: "COPPER-WIRE".into(),
                    warehouse: "Stores - ACME".into(),
                    qty: dec!(120.0),
                    valuation_rate: dec!(10.0),
                    current_qty: Decimal::ZERO,
                    current_valuation_rate: Decimal::ZERO,
                    quantity_difference: Decimal::ZERO,
                    amount_difference: Decimal::ZERO,
                    serial_nos: vec![],
                    batch_no: None,
                },
                // Item 2: Physical count found 40 (system had 50 @ $20 = $1,000). Reconciled @ $20 = $800. -$200 Loss.
                StockReconciliationItem {
                    item_code: "ALUMINUM-SHEET".into(),
                    warehouse: "Stores - ACME".into(),
                    qty: dec!(40.0),
                    valuation_rate: dec!(20.0),
                    current_qty: Decimal::ZERO,
                    current_valuation_rate: Decimal::ZERO,
                    quantity_difference: Decimal::ZERO,
                    amount_difference: Decimal::ZERO,
                    serial_nos: vec![],
                    batch_no: None,
                },
            ],
            total_amount_difference: Decimal::ZERO,
            docstatus: 0,
            remarks: Some("Annual physical inventory count".into()),
        };

        // Submit with system lookup mock
        rec.submit(|item, _wh| match item {
            "COPPER-WIRE" => (dec!(100.0), dec!(10.0)),
            "ALUMINUM-SHEET" => (dec!(50.0), dec!(20.0)),
            _ => (Decimal::ZERO, Decimal::ZERO),
        })
        .expect("Stock reconciliation submit failed");

        assert_eq!(rec.docstatus, 1);
        assert_eq!(rec.items[0].quantity_difference, dec!(20.0));
        assert_eq!(rec.items[0].amount_difference, dec!(200.00));
        assert_eq!(rec.items[1].quantity_difference, dec!(-10.0));
        assert_eq!(rec.items[1].amount_difference, dec!(-200.00));
        assert_eq!(rec.total_amount_difference, dec!(0.00));

        // Generate SLEs
        let sles = rec.generate_stock_ledger_entries();
        assert_eq!(sles.len(), 2);
        assert_eq!(sles[0].actual_qty, dec!(20.0));
        assert_eq!(sles[0].stock_value_difference, dec!(200.00));
        assert_eq!(sles[1].actual_qty, dec!(-10.0));
        assert_eq!(sles[1].stock_value_difference, dec!(-200.00));

        // Generate GL entries
        let gl_entries = rec
            .generate_gl_entries(|wh| match wh {
                "Stores - ACME" => Ok("1300 - Stock In Hand".into()),
                _ => Err(StockReconciliationError::WarehouseAccountNotFound(
                    wh.into(),
                )),
            })
            .expect("GL entries generation failed");

        assert_eq!(gl_entries.len(), 4);
        let total_debit: Decimal = gl_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = gl_entries.iter().map(|e| e.credit).sum();
        assert_eq!(total_debit, total_credit);
        assert_eq!(total_debit, dec!(400.00));
    }
}
