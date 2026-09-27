use chrono::NaiveDate;
use erp_accounting::{GlEntry, JournalEntry, JournalEntryLine};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Warehouse master record with linked General Ledger accounts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Warehouse {
    /// Warehouse identifier (e.g. "Finished Goods - ACME").
    pub name: String,
    /// Linked balance sheet asset account for inventory on-hand.
    pub stock_in_hand_account: String,
    /// Linked liability account for goods received but pending invoice.
    pub stock_received_but_not_billed_account: String,
    /// Linked expense account for cost of goods sold.
    pub default_cogs_account: String,
    /// Company identifier.
    pub company: String,
}

/// Generates balancing GL entries for a Purchase Receipt submission (Milestone 2.7).
/// - Debit: Stock In Hand (Asset increases)
/// - Credit: Stock Received But Not Billed (Liability increases)
#[must_use]
pub fn create_purchase_receipt_gl_entries(
    warehouse: &Warehouse,
    total_valuation_amount: Decimal,
    posting_date: NaiveDate,
    voucher_no: &str,
) -> Vec<GlEntry> {
    vec![
        GlEntry {
            name: format!("{voucher_no}-1"),
            posting_date,
            account: warehouse.stock_in_hand_account.clone(),
            debit: total_valuation_amount,
            credit: Decimal::ZERO,
            voucher_type: "Purchase Receipt".to_string(),
            voucher_no: voucher_no.to_string(),
            party_type: None,
            party: None,
            company: warehouse.company.clone(),
        },
        GlEntry {
            name: format!("{voucher_no}-2"),
            posting_date,
            account: warehouse.stock_received_but_not_billed_account.clone(),
            debit: Decimal::ZERO,
            credit: total_valuation_amount,
            voucher_type: "Purchase Receipt".to_string(),
            voucher_no: voucher_no.to_string(),
            party_type: None,
            party: None,
            company: warehouse.company.clone(),
        },
    ]
}

/// Generates balancing GL entries for a Delivery Note submission (Milestone 2.7).
/// - Debit: Cost of Goods Sold (Expense increases)
/// - Credit: Stock In Hand (Asset decreases)
#[must_use]
pub fn create_delivery_note_gl_entries(
    warehouse: &Warehouse,
    cogs_amount: Decimal,
    posting_date: NaiveDate,
    voucher_no: &str,
) -> Vec<GlEntry> {
    vec![
        GlEntry {
            name: format!("{voucher_no}-1"),
            posting_date,
            account: warehouse.default_cogs_account.clone(),
            debit: cogs_amount,
            credit: Decimal::ZERO,
            voucher_type: "Delivery Note".to_string(),
            voucher_no: voucher_no.to_string(),
            party_type: None,
            party: None,
            company: warehouse.company.clone(),
        },
        GlEntry {
            name: format!("{voucher_no}-2"),
            posting_date,
            account: warehouse.stock_in_hand_account.clone(),
            debit: Decimal::ZERO,
            credit: cogs_amount,
            voucher_type: "Delivery Note".to_string(),
            voucher_no: voucher_no.to_string(),
            party_type: None,
            party: None,
            company: warehouse.company.clone(),
        },
    ]
}

/// Helper converting GL entries into a validated JournalEntry document.
#[must_use]
pub fn gl_entries_to_journal_entry(
    entries: &[GlEntry],
    posting_date: NaiveDate,
    company: &str,
    remarks: &str,
) -> JournalEntry {
    let lines = entries
        .iter()
        .map(|e| JournalEntryLine {
            account: e.account.clone(),
            debit: e.debit,
            credit: e.credit,
            debit_in_account_currency: e.debit,
            credit_in_account_currency: e.credit,
            exchange_rate: Decimal::ONE,
            party_type: e.party_type.clone(),
            party: e.party.clone(),
        })
        .collect();

    JournalEntry {
        posting_date,
        company: company.to_string(),
        lines,
        remarks: remarks.to_string(),
    }
}
