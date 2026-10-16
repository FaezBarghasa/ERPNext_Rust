//! Transactional Payment Entry Controller (`erp-accounting::payment_entry`).
//!
//! Implements canonical upstream ERPNext Payment Entry business logic:
//! - Payment types: `Receive` (Customer), `Pay` (Supplier), `InternalTransfer` (Bank to Cash)
//! - Multi-invoice allocation table with partial/full settlements
//! - Automated Foreign Exchange (Forex) Gain/Loss calculation and journal entry balancing
//! - Deduction and Tax Withholding (TDS, Bank Fees) allocation
//! - Bidirectional Payment Ledger Entry (PLE) generation with zero-loss precision.

use crate::ledger::{AccountingError, JournalEntryLine};
use crate::payment_ledger::PaymentLedgerEntry;
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Payment Entry domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PaymentEntryError {
    #[error(
        "Paid amount and received amount must be positive, got paid: {paid}, received: {received}"
    )]
    InvalidAmounts { paid: Decimal, received: Decimal },
    #[error("Party and Party Type are required for '{0:?}' payment type")]
    PartyRequired(PaymentType),
    #[error("Total allocated amount ({allocated}) exceeds payment capacity ({capacity})")]
    OverAllocation {
        allocated: Decimal,
        capacity: Decimal,
    },
    #[error("Cannot submit cancelled or draft document (current status: {0})")]
    InvalidDocStatus(i32),
    #[error("Accounting error during GL entry generation: {0}")]
    Accounting(#[from] AccountingError),
}

/// Type of payment entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentType {
    /// Inward payment from a Customer or Debtor.
    Receive,
    /// Outward disbursement to a Supplier, Vendor, or Employee.
    Pay,
    /// Fund transfer between internal Company bank/cash accounts.
    InternalTransfer,
}

/// An individual invoice or order allocation line in Payment Entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentEntryReference {
    pub reference_doctype: CompactString, // "Sales Invoice", "Purchase Invoice"
    pub reference_name: CompactString,
    pub total_amount: Decimal,
    pub outstanding_amount: Decimal,
    pub allocated_amount: Decimal,
    pub exchange_rate: Decimal, // Historical exchange rate of the invoice
    pub exchange_gain_loss: Decimal,
}

/// Deduction or charge line (e.g. Bank Service Fee, TDS Withholding).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentEntryDeduction {
    pub account: CompactString,
    pub cost_center: Option<CompactString>,
    pub amount: Decimal,
}

/// Canonical ERPNext Payment Entry document entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentEntry {
    pub name: CompactString,
    pub payment_type: PaymentType,
    pub posting_date: NaiveDate,
    pub company: CompactString,
    pub party_type: Option<CompactString>,
    pub party: Option<CompactString>,
    pub paid_from: CompactString,
    pub paid_to: CompactString,
    pub paid_amount: Decimal,
    pub received_amount: Decimal,
    pub source_exchange_rate: Decimal,
    pub target_exchange_rate: Decimal,
    pub references: Vec<PaymentEntryReference>,
    pub deductions: Vec<PaymentEntryDeduction>,
    pub remarks: Option<String>,
    pub docstatus: i32, // 0 = Draft, 1 = Submitted, 2 = Cancelled
}

impl PaymentEntry {
    /// Validates payment invariant rules, party requirements, and allocations.
    pub fn validate(&mut self) -> Result<(), PaymentEntryError> {
        if self.paid_amount <= Decimal::ZERO || self.received_amount <= Decimal::ZERO {
            return Err(PaymentEntryError::InvalidAmounts {
                paid: self.paid_amount,
                received: self.received_amount,
            });
        }

        if matches!(self.payment_type, PaymentType::Receive | PaymentType::Pay)
            && (self.party.is_none() || self.party_type.is_none())
        {
            return Err(PaymentEntryError::PartyRequired(self.payment_type));
        }

        let total_allocated: Decimal = self.references.iter().map(|r| r.allocated_amount).sum();
        let total_deductions: Decimal = self.deductions.iter().map(|d| d.amount).sum();

        let base_capacity = match self.payment_type {
            PaymentType::Receive => self.received_amount + total_deductions,
            PaymentType::Pay => self.paid_amount,
            PaymentType::InternalTransfer => self.paid_amount,
        };

        if total_allocated > base_capacity {
            return Err(PaymentEntryError::OverAllocation {
                allocated: total_allocated,
                capacity: base_capacity,
            });
        }

        // Calculate Forex Exchange Gain/Loss per reference:
        // Gain/Loss = allocated_amount * (current_settlement_rate - original_invoice_rate)
        for ref_line in &mut self.references {
            let current_rate = match self.payment_type {
                PaymentType::Receive => self.target_exchange_rate,
                PaymentType::Pay => self.source_exchange_rate,
                PaymentType::InternalTransfer => Decimal::ONE,
            };

            let rate_diff = current_rate - ref_line.exchange_rate;
            ref_line.exchange_gain_loss = ref_line.allocated_amount * rate_diff;
        }

        Ok(())
    }

    /// Submits the Payment Entry, transitioning status from Draft (0) to Submitted (1).
    pub fn submit(&mut self) -> Result<(), PaymentEntryError> {
        if self.docstatus != 0 {
            return Err(PaymentEntryError::InvalidDocStatus(self.docstatus));
        }
        self.validate()?;
        self.docstatus = 1;
        Ok(())
    }

    /// Generates strictly balanced General Ledger journal lines with automated exchange gain/loss booking.
    ///
    /// Invariant: $\sum \text{Debit} - \sum \text{Credit} = 0$
    pub fn generate_gl_entries(
        &self,
        forex_account: &str,
    ) -> Result<Vec<JournalEntryLine>, PaymentEntryError> {
        let mut lines = Vec::new();

        match self.payment_type {
            PaymentType::Receive => {
                // 1. Debit Bank/Cash receiving account in base currency
                let bank_base = self.received_amount * self.target_exchange_rate;
                lines.push(JournalEntryLine {
                    account: self.paid_to.to_string(),
                    debit: bank_base,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: self.received_amount,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: self.target_exchange_rate,
                    party_type: None,
                    party: None,
                });

                // 2. Debit any deductions (e.g. Bank fees, TDS withholding)
                for ded in &self.deductions {
                    lines.push(JournalEntryLine {
                        account: ded.account.to_string(),
                        debit: ded.amount,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: ded.amount,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                }

                // 3. Credit Customer Receivable account per invoice allocation
                let mut total_credited = Decimal::ZERO;
                for ref_line in &self.references {
                    let base_credit = ref_line.allocated_amount * ref_line.exchange_rate;
                    total_credited += base_credit;

                    lines.push(JournalEntryLine {
                        account: self.paid_from.to_string(),
                        debit: Decimal::ZERO,
                        credit: base_credit,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: ref_line.allocated_amount,
                        exchange_rate: ref_line.exchange_rate,
                        party_type: self.party_type.as_ref().map(|s| s.to_string()),
                        party: self.party.as_ref().map(|s| s.to_string()),
                    });
                }

                // 4. Handle unallocated advance payment
                let total_allocated: Decimal =
                    self.references.iter().map(|r| r.allocated_amount).sum();
                let unallocated = self.received_amount - total_allocated;
                if unallocated > Decimal::ZERO {
                    let unallocated_base = unallocated * self.target_exchange_rate;
                    total_credited += unallocated_base;

                    lines.push(JournalEntryLine {
                        account: self.paid_from.to_string(),
                        debit: Decimal::ZERO,
                        credit: unallocated_base,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: unallocated,
                        exchange_rate: self.target_exchange_rate,
                        party_type: self.party_type.as_ref().map(|s| s.to_string()),
                        party: self.party.as_ref().map(|s| s.to_string()),
                    });
                }

                // 5. Balance Foreign Exchange Difference
                let total_debit: Decimal = lines.iter().map(|l| l.debit).sum();
                let total_credit: Decimal = lines.iter().map(|l| l.credit).sum();

                if total_debit > total_credit {
                    // Net Exchange Gain (Credit Forex Account)
                    lines.push(JournalEntryLine {
                        account: forex_account.to_string(),
                        debit: Decimal::ZERO,
                        credit: total_debit - total_credit,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: total_debit - total_credit,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                } else if total_credit > total_debit {
                    // Net Exchange Loss (Debit Forex Account)
                    lines.push(JournalEntryLine {
                        account: forex_account.to_string(),
                        debit: total_credit - total_debit,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: total_credit - total_debit,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                }
            }

            PaymentType::Pay => {
                // 1. Debit Creditor/Supplier Account per invoice allocation
                let mut total_debited = Decimal::ZERO;
                for ref_line in &self.references {
                    let base_debit = ref_line.allocated_amount * ref_line.exchange_rate;
                    total_debited += base_debit;

                    lines.push(JournalEntryLine {
                        account: self.paid_to.to_string(),
                        debit: base_debit,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: ref_line.allocated_amount,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: ref_line.exchange_rate,
                        party_type: self.party_type.as_ref().map(|s| s.to_string()),
                        party: self.party.as_ref().map(|s| s.to_string()),
                    });
                }

                // Unallocated advance
                let total_allocated: Decimal =
                    self.references.iter().map(|r| r.allocated_amount).sum();
                let unallocated = self.paid_amount - total_allocated;
                if unallocated > Decimal::ZERO {
                    let unallocated_base = unallocated * self.source_exchange_rate;
                    total_debited += unallocated_base;

                    lines.push(JournalEntryLine {
                        account: self.paid_to.to_string(),
                        debit: unallocated_base,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: unallocated,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: self.source_exchange_rate,
                        party_type: self.party_type.as_ref().map(|s| s.to_string()),
                        party: self.party.as_ref().map(|s| s.to_string()),
                    });
                }

                // 2. Credit Deductions (e.g. TDS withholding payable)
                let total_deductions: Decimal = self.deductions.iter().map(|d| d.amount).sum();
                for ded in &self.deductions {
                    lines.push(JournalEntryLine {
                        account: ded.account.to_string(),
                        debit: Decimal::ZERO,
                        credit: ded.amount,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: ded.amount,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                }

                // 3. Credit Bank/Cash Account (net disbursed after deductions)
                let net_paid_amount = self.paid_amount - total_deductions;
                let bank_base = net_paid_amount * self.source_exchange_rate;
                lines.push(JournalEntryLine {
                    account: self.paid_from.to_string(),
                    debit: Decimal::ZERO,
                    credit: bank_base,
                    debit_in_account_currency: Decimal::ZERO,
                    credit_in_account_currency: net_paid_amount,
                    exchange_rate: self.source_exchange_rate,
                    party_type: None,
                    party: None,
                });

                // 3. Balance Forex
                let total_debit: Decimal = lines.iter().map(|l| l.debit).sum();
                let total_credit: Decimal = lines.iter().map(|l| l.credit).sum();

                if total_debit > total_credit {
                    lines.push(JournalEntryLine {
                        account: forex_account.to_string(),
                        debit: Decimal::ZERO,
                        credit: total_debit - total_credit,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: total_debit - total_credit,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                } else if total_credit > total_debit {
                    lines.push(JournalEntryLine {
                        account: forex_account.to_string(),
                        debit: total_credit - total_debit,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: total_credit - total_debit,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    });
                }
            }

            PaymentType::InternalTransfer => {
                let from_base = self.paid_amount * self.source_exchange_rate;
                let to_base = self.received_amount * self.target_exchange_rate;

                lines.push(JournalEntryLine {
                    account: self.paid_to.to_string(),
                    debit: to_base,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: self.received_amount,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: self.target_exchange_rate,
                    party_type: None,
                    party: None,
                });

                lines.push(JournalEntryLine {
                    account: self.paid_from.to_string(),
                    debit: Decimal::ZERO,
                    credit: from_base,
                    debit_in_account_currency: Decimal::ZERO,
                    credit_in_account_currency: self.paid_amount,
                    exchange_rate: self.source_exchange_rate,
                    party_type: None,
                    party: None,
                });
            }
        }

        // Validate final balance
        let final_debit: Decimal = lines.iter().map(|l| l.debit).sum();
        let final_credit: Decimal = lines.iter().map(|l| l.credit).sum();
        if final_debit != final_credit {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit: final_debit,
                total_credit: final_credit,
            }
            .into());
        }

        Ok(lines)
    }

    /// Generates Payment Ledger Entries (PLE) to reconcile outstanding invoice balances.
    #[must_use]
    pub fn generate_ple_entries(&self) -> Vec<PaymentLedgerEntry> {
        let mut ple_entries = Vec::new();

        let (account_type, party_account) = match self.payment_type {
            PaymentType::Receive => ("Receivable", self.paid_from.clone()),
            PaymentType::Pay => ("Payable", self.paid_to.clone()),
            PaymentType::InternalTransfer => return ple_entries,
        };

        let party_type = self.party_type.clone().unwrap_or_else(|| "Customer".into());
        let party = self.party.clone().unwrap_or_else(|| "Unknown".into());

        for (idx, ref_line) in self.references.iter().enumerate() {
            // Negative amount reduces invoice outstanding
            let ple_amount = match self.payment_type {
                PaymentType::Receive => -ref_line.allocated_amount * ref_line.exchange_rate,
                PaymentType::Pay => -ref_line.allocated_amount * ref_line.exchange_rate,
                PaymentType::InternalTransfer => Decimal::ZERO,
            };

            ple_entries.push(PaymentLedgerEntry {
                name: format!("{}-{}", self.name, idx + 1).into(),
                company: self.company.clone(),
                posting_date: self.posting_date,
                account_type: account_type.into(),
                party_type: party_type.clone(),
                party: party.clone(),
                account: party_account.clone(),
                voucher_type: "Payment Entry".into(),
                voucher_no: self.name.clone(),
                against_voucher_type: Some(ref_line.reference_doctype.clone()),
                against_voucher_no: Some(ref_line.reference_name.clone()),
                amount: ple_amount,
                amount_in_account_currency: -ref_line.allocated_amount,
                currency: "USD".into(),
                delinked: false,
            });
        }

        // Handle advance unallocated component
        let total_allocated: Decimal = self.references.iter().map(|r| r.allocated_amount).sum();
        let unallocated = match self.payment_type {
            PaymentType::Receive => self.received_amount - total_allocated,
            PaymentType::Pay => self.paid_amount - total_allocated,
            PaymentType::InternalTransfer => Decimal::ZERO,
        };

        if unallocated > Decimal::ZERO {
            let rate = match self.payment_type {
                PaymentType::Receive => self.target_exchange_rate,
                PaymentType::Pay => self.source_exchange_rate,
                PaymentType::InternalTransfer => Decimal::ONE,
            };

            ple_entries.push(PaymentLedgerEntry {
                name: format!("{}-ADV", self.name).into(),
                company: self.company.clone(),
                posting_date: self.posting_date,
                account_type: account_type.into(),
                party_type,
                party,
                account: party_account,
                voucher_type: "Payment Entry".into(),
                voucher_no: self.name.clone(),
                against_voucher_type: None,
                against_voucher_no: None,
                amount: -unallocated * rate,
                amount_in_account_currency: -unallocated,
                currency: "USD".into(),
                delinked: false,
            });
        }

        ple_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payment_ledger::PaymentLedgerEngine;
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_entry_receive_with_forex_gain_and_deductions() {
        let mut pe = PaymentEntry {
            name: "ACC-PAY-2026-0001".into(),
            payment_type: PaymentType::Receive,
            posting_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            company: "Enterprise Global Corp".into(),
            party_type: Some("Customer".into()),
            party: Some("CUST-001".into()),
            paid_from: "Debtors - Enterprise".into(),
            paid_to: "HDFC Bank - Enterprise".into(),
            paid_amount: dec!(1000.0),     // Invoice currency: $1,000 USD
            received_amount: dec!(1000.0), // Bank received: $1,000 USD
            source_exchange_rate: dec!(82.0),
            target_exchange_rate: dec!(85.0), // Rupee depreciated -> Forex Gain of 3 INR per USD
            references: vec![PaymentEntryReference {
                reference_doctype: "Sales Invoice".into(),
                reference_name: "ACC-SINV-2026-0001".into(),
                total_amount: dec!(1000.0),
                outstanding_amount: dec!(1000.0),
                allocated_amount: dec!(1000.0),
                exchange_rate: dec!(82.0), // Invoice was booked at 82.0
                exchange_gain_loss: Decimal::ZERO,
            }],
            deductions: vec![PaymentEntryDeduction {
                account: "Bank Charges - Enterprise".into(),
                cost_center: None,
                amount: dec!(15.0), // $15 bank wire fee
            }],
            remarks: Some("Settlement of SINV-0001".into()),
            docstatus: 0,
        };

        pe.submit().expect("Payment Entry submit should succeed");
        assert_eq!(pe.docstatus, 1);
        assert_eq!(pe.references[0].exchange_gain_loss, dec!(3000.0));

        let gl_entries = pe
            .generate_gl_entries("Exchange Gain/Loss - Enterprise")
            .expect("GL entries should balance");

        let total_debit: Decimal = gl_entries.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = gl_entries.iter().map(|l| l.credit).sum();

        // Bank debit = 85,000 + 15 fee = 85,015
        // Customer credit = 82,000
        // Forex Gain credit = 3,015
        assert_eq!(total_debit, total_credit);
        assert_eq!(total_debit, dec!(85015.0));

        // Reconcile with PaymentLedgerEngine
        let mut ple_engine = PaymentLedgerEngine::new();
        // 1. Initial sales invoice PLE
        ple_engine.record_entry(PaymentLedgerEntry {
            name: "PLE-INV-1".into(),
            company: "Enterprise Global Corp".into(),
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            account_type: "Receivable".into(),
            party_type: "Customer".into(),
            party: "CUST-001".into(),
            account: "Debtors - Enterprise".into(),
            voucher_type: "Sales Invoice".into(),
            voucher_no: "ACC-SINV-2026-0001".into(),
            against_voucher_type: None,
            against_voucher_no: None,
            amount: dec!(82000.0),
            amount_in_account_currency: dec!(1000.0),
            currency: "USD".into(),
            delinked: false,
        });

        // 2. Post payment entry PLEs
        let ples = pe.generate_ple_entries();
        ple_engine.record_entries(ples);

        // 3. Verify outstanding is now 0.00
        let outstanding = ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-2026-0001");
        assert_eq!(outstanding, Decimal::ZERO);
    }
}
