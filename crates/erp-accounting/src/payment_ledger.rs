//! Payment Ledger Entry (PLE) & AR/AP Reconciliation Engine (`erp-accounting::payment_ledger`).
//!
//! Tracks granular invoice allocations, advance payment allocations, outstanding invoice calculations,
//! and bidirectional payment reconciliation / unreconciliation without floating-point drift.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An individual Payment Ledger Entry (PLE) row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentLedgerEntry {
    pub name: CompactString,
    pub company: CompactString,
    pub posting_date: NaiveDate,
    pub account_type: CompactString, // "Receivable" or "Payable"
    pub party_type: CompactString,   // "Customer" or "Supplier"
    pub party: CompactString,
    pub account: CompactString,
    pub voucher_type: CompactString, // "Sales Invoice", "Payment Entry", "Journal Entry"
    pub voucher_no: CompactString,
    pub against_voucher_type: Option<CompactString>,
    pub against_voucher_no: Option<CompactString>,
    pub amount: Decimal, // Positive for invoice, negative for payment/credit note
    pub amount_in_account_currency: Decimal,
    pub currency: CompactString,
    pub delinked: bool,
}

/// Payment Ledger & Settlement Reconciliation Engine.
#[derive(Default)]
pub struct PaymentLedgerEngine {
    entries: Vec<PaymentLedgerEntry>,
}

impl PaymentLedgerEngine {
    /// Creates a new in-memory payment ledger engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new Payment Ledger Entry.
    pub fn record_entry(&mut self, entry: PaymentLedgerEntry) {
        self.entries.push(entry);
    }

    /// Records multiple Payment Ledger Entries atomically.
    pub fn record_entries(&mut self, entries: impl IntoIterator<Item = PaymentLedgerEntry>) {
        self.entries.extend(entries);
    }

    /// Calculates the current outstanding balance for a given invoice voucher.
    ///
    /// $$\text{Outstanding} = \sum_{\text{voucher}=\text{inv}} \text{amount} + \sum_{\text{against}=\text{inv}} \text{amount}$$
    #[must_use]
    pub fn calculate_outstanding(&self, voucher_type: &str, voucher_no: &str) -> Decimal {
        let mut balance = Decimal::ZERO;

        for ple in &self.entries {
            if ple.delinked {
                continue;
            }

            // Direct invoice creation entry (positive)
            if ple.voucher_type == voucher_type
                && ple.voucher_no == voucher_no
                && ple.against_voucher_no.is_none()
            {
                balance += ple.amount;
            }
            // Payment or Credit Note applied against this invoice (negative)
            else if ple.against_voucher_type.as_deref() == Some(voucher_type)
                && ple.against_voucher_no.as_deref() == Some(voucher_no)
            {
                balance += ple.amount; // amount is negative for payment
            }
        }

        balance
    }

    /// Calculates total outstanding balance for a party across all open invoices.
    #[must_use]
    pub fn calculate_party_balance(&self, party_type: &str, party: &str) -> Decimal {
        self.entries
            .iter()
            .filter(|ple| !ple.delinked && ple.party_type == party_type && ple.party == party)
            .map(|ple| ple.amount)
            .sum()
    }

    /// Reconciles an unallocated Payment Entry against an open Invoice.
    pub fn reconcile(
        &mut self,
        payment_voucher_no: &str,
        invoice_voucher_type: &str,
        invoice_voucher_no: &str,
        allocated_amount: Decimal,
        posting_date: NaiveDate,
    ) -> Result<PaymentLedgerEntry, &'static str> {
        if allocated_amount <= Decimal::ZERO {
            return Err("Allocated amount must be strictly positive");
        }

        // Find reference payment entry to inherit party, company, account, currency
        let base_payment = self
            .entries
            .iter()
            .find(|ple| ple.voucher_no == payment_voucher_no)
            .cloned()
            .ok_or("Payment voucher not found in Payment Ledger")?;

        let current_outstanding =
            self.calculate_outstanding(invoice_voucher_type, invoice_voucher_no);
        if allocated_amount > current_outstanding {
            return Err("Allocation exceeds open invoice outstanding amount");
        }

        let ple_name = format!("PLE-{}-REC", self.entries.len() + 1);
        let allocation_entry = PaymentLedgerEntry {
            name: CompactString::new(&ple_name),
            company: base_payment.company,
            posting_date,
            account_type: base_payment.account_type,
            party_type: base_payment.party_type,
            party: base_payment.party,
            account: base_payment.account,
            voucher_type: base_payment.voucher_type,
            voucher_no: CompactString::new(payment_voucher_no),
            against_voucher_type: Some(CompactString::new(invoice_voucher_type)),
            against_voucher_no: Some(CompactString::new(invoice_voucher_no)),
            amount: -allocated_amount, // Negative allocation reduces invoice balance
            amount_in_account_currency: -allocated_amount,
            currency: base_payment.currency,
            delinked: false,
        };

        self.record_entry(allocation_entry.clone());
        Ok(allocation_entry)
    }

    /// Unreconciles a payment allocation, delinking matching entries.
    pub fn unreconcile(&mut self, payment_voucher_no: &str, invoice_voucher_no: &str) -> usize {
        let mut count = 0;
        for ple in &mut self.entries {
            if ple.voucher_no == payment_voucher_no
                && ple.against_voucher_no.as_deref() == Some(invoice_voucher_no)
                && !ple.delinked
            {
                ple.delinked = true;
                count += 1;
            }
        }
        count
    }

    /// Fetches all active entries for a voucher.
    #[must_use]
    pub fn get_entries_for_voucher(&self, voucher_no: &str) -> Vec<PaymentLedgerEntry> {
        self.entries
            .iter()
            .filter(|ple| {
                !ple.delinked
                    && (ple.voucher_no == voucher_no
                        || ple.against_voucher_no.as_deref() == Some(voucher_no))
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_ledger_reconciliation_and_outstanding_lifecycle() {
        let mut ple_engine = PaymentLedgerEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        // 1. Post Sales Invoice for $1,000
        ple_engine.record_entry(PaymentLedgerEntry {
            name: "PLE-001".into(),
            company: "Wind Energy Corp".into(),
            posting_date: date,
            account_type: "Receivable".into(),
            party_type: "Customer".into(),
            party: "CUST-001".into(),
            account: "1200 - Debtors".into(),
            voucher_type: "Sales Invoice".into(),
            voucher_no: "ACC-SINV-001".into(),
            against_voucher_type: None,
            against_voucher_no: None,
            amount: dec!(1000.00),
            amount_in_account_currency: dec!(1000.00),
            currency: "USD".into(),
            delinked: false,
        });

        assert_eq!(
            ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
            dec!(1000.00)
        );

        // 2. Post Unallocated Payment Entry of $600
        ple_engine.record_entry(PaymentLedgerEntry {
            name: "PLE-002".into(),
            company: "Wind Energy Corp".into(),
            posting_date: date,
            account_type: "Receivable".into(),
            party_type: "Customer".into(),
            party: "CUST-001".into(),
            account: "1200 - Debtors".into(),
            voucher_type: "Payment Entry".into(),
            voucher_no: "ACC-PAY-001".into(),
            against_voucher_type: None,
            against_voucher_no: None,
            amount: dec!(-600.00),
            amount_in_account_currency: dec!(-600.00),
            currency: "USD".into(),
            delinked: false,
        });

        // 3. Reconcile $600 against Sales Invoice
        ple_engine
            .reconcile(
                "ACC-PAY-001",
                "Sales Invoice",
                "ACC-SINV-001",
                dec!(600.00),
                date,
            )
            .unwrap();

        assert_eq!(
            ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
            dec!(400.00)
        );

        // 4. Over-allocation check (attempting to allocate $500 when only $400 is left)
        let err = ple_engine.reconcile(
            "ACC-PAY-001",
            "Sales Invoice",
            "ACC-SINV-001",
            dec!(500.00),
            date,
        );
        assert!(err.is_err());

        // 5. Unreconcile payment
        let count = ple_engine.unreconcile("ACC-PAY-001", "ACC-SINV-001");
        assert_eq!(count, 1);
        assert_eq!(
            ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
            dec!(1000.00)
        );
    }
}
