use crate::coa::{Account, RootType};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Accounting domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountingError {
    /// Transaction debits and credits do not balance to zero.
    #[error(
        "Unbalanced transaction: Total Debit ({total_debit}) != Total Credit ({total_credit})"
    )]
    UnbalancedTransaction {
        total_debit: Decimal,
        total_credit: Decimal,
    },
    /// Attempted to post into a closed fiscal period.
    #[error("Cannot post to closed period on or before {0}")]
    PeriodClosed(NaiveDate),
    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    /// Customer credit limit exceeded.
    #[error(
        "Credit limit exceeded: Outstanding ({current}) + New ({new_amount}) > Limit ({limit})"
    )]
    CreditLimitExceeded {
        current: Decimal,
        new_amount: Decimal,
        limit: Decimal,
    },
    /// Invalid exchange rate.
    #[error("Exchange rate must be positive, got {0}")]
    InvalidExchangeRate(Decimal),
}

/// An individual debit/credit line item in a Journal Entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntryLine {
    /// Target general ledger account.
    pub account: String,
    /// Debit amount in company base currency.
    pub debit: Decimal,
    /// Credit amount in company base currency.
    pub credit: Decimal,
    /// Debit amount in account original currency.
    pub debit_in_account_currency: Decimal,
    /// Credit amount in account original currency.
    pub credit_in_account_currency: Decimal,
    /// Exchange rate relative to company base currency.
    pub exchange_rate: Decimal,
    /// Associated party type (e.g. "Customer", "Supplier").
    pub party_type: Option<String>,
    /// Associated party identifier (e.g. "CUST-0001").
    pub party: Option<String>,
}

/// A complete journal entry transaction document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Company identifier.
    pub company: String,
    /// Transaction line items.
    pub lines: Vec<JournalEntryLine>,
    /// User remarks / narrative.
    pub remarks: String,
}

impl JournalEntry {
    /// Validates the zero-loss balancing equation $\sum \text{Debit} - \sum \text{Credit} = 0$.
    pub fn validate_balance(&self) -> Result<(), AccountingError> {
        let total_debit: Decimal = self.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = self.lines.iter().map(|l| l.credit).sum();

        if total_debit != total_credit {
            return Err(AccountingError::UnbalancedTransaction {
                total_debit,
                total_credit,
            });
        }

        for line in &self.lines {
            if line.exchange_rate <= Decimal::ZERO {
                return Err(AccountingError::InvalidExchangeRate(line.exchange_rate));
            }
        }

        Ok(())
    }

    /// Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.
    pub fn balance_exchange_variance(
        &mut self,
        gain_loss_account: &str,
    ) -> Result<(), AccountingError> {
        let total_debit: Decimal = self.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = self.lines.iter().map(|l| l.credit).sum();
        let diff = total_debit - total_credit;

        if diff != Decimal::ZERO {
            if diff > Decimal::ZERO {
                // Credit Gain
                self.lines.push(JournalEntryLine {
                    account: gain_loss_account.to_string(),
                    debit: Decimal::ZERO,
                    credit: diff,
                    debit_in_account_currency: Decimal::ZERO,
                    credit_in_account_currency: diff,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                });
            } else {
                // Debit Loss
                let loss = diff.abs();
                self.lines.push(JournalEntryLine {
                    account: gain_loss_account.to_string(),
                    debit: loss,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: loss,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                });
            }
        }

        self.validate_balance()
    }
}

/// Immutable General Ledger Entry row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlEntry {
    /// Primary key identifier.
    pub name: String,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Target posting account.
    pub account: String,
    /// Debit amount in company base currency.
    pub debit: Decimal,
    /// Credit amount in company base currency.
    pub credit: Decimal,
    /// Originating transaction document type.
    pub voucher_type: String,
    /// Originating transaction document identifier.
    pub voucher_no: String,
    /// Associated party type.
    pub party_type: Option<String>,
    /// Associated party identifier.
    pub party: Option<String>,
    /// Company identifier.
    pub company: String,
}

/// Period closing record sealing fiscal transactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodClosingLog {
    /// Company name.
    pub company: String,
    /// Fiscal period closing date (all postings on or prior are locked).
    pub closing_date: NaiveDate,
}

/// Atomic Ledger Posting Pipeline.
pub struct LedgerPostingEngine {
    closing_logs: Vec<PeriodClosingLog>,
    accounts: HashMap<String, Account>,
}

impl LedgerPostingEngine {
    /// Creates a new posting engine instance.
    #[must_use]
    pub fn new(accounts: Vec<Account>, closing_logs: Vec<PeriodClosingLog>) -> Self {
        let mut acct_map = HashMap::new();
        for a in accounts {
            acct_map.insert(a.name.clone(), a);
        }
        Self {
            closing_logs,
            accounts: acct_map,
        }
    }

    /// Posts a journal entry to the immutable general ledger.
    pub fn post_journal_entry(
        &self,
        entry: &JournalEntry,
        voucher_no: &str,
    ) -> Result<Vec<GlEntry>, AccountingError> {
        // 1. Period Closed Validation (Milestone 2.3)
        for log in &self.closing_logs {
            if log.company == entry.company && entry.posting_date <= log.closing_date {
                return Err(AccountingError::PeriodClosed(log.closing_date));
            }
        }

        // 2. Balancing Validation (Milestone 2.1)
        entry.validate_balance()?;

        // 3. Emit immutable GL Entries
        let mut gl_entries = Vec::new();
        for (i, line) in entry.lines.iter().enumerate() {
            if !self.accounts.contains_key(&line.account) {
                return Err(AccountingError::AccountNotFound(line.account.clone()));
            }

            gl_entries.push(GlEntry {
                name: format!("{voucher_no}-{i}"),
                posting_date: entry.posting_date,
                account: line.account.clone(),
                debit: line.debit,
                credit: line.credit,
                voucher_type: "Journal Entry".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: line.party_type.clone(),
                party: line.party.clone(),
                company: entry.company.clone(),
            });
        }

        Ok(gl_entries)
    }
}

/// Trial Balance row summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrialBalanceRow {
    pub account: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balance: Decimal,
}

/// Real-time Financial Statement Generator (Milestone 2.4).
pub struct StatementGenerator;

impl StatementGenerator {
    /// Generates real-time Trial Balance.
    #[must_use]
    pub fn generate_trial_balance(entries: &[GlEntry], as_of: NaiveDate) -> Vec<TrialBalanceRow> {
        let mut account_totals: HashMap<String, (Decimal, Decimal)> = HashMap::new();

        for e in entries {
            if e.posting_date <= as_of {
                let entry = account_totals
                    .entry(e.account.clone())
                    .or_insert((Decimal::ZERO, Decimal::ZERO));
                entry.0 += e.debit;
                entry.1 += e.credit;
            }
        }

        let mut rows: Vec<TrialBalanceRow> = account_totals
            .into_iter()
            .map(|(account, (total_debit, total_credit))| TrialBalanceRow {
                account,
                total_debit,
                total_credit,
                balance: total_debit - total_credit,
            })
            .collect();

        rows.sort_by(|a, b| a.account.cmp(&b.account));
        rows
    }

    /// Computes Net Profit ($\sum \text{Income} - \sum \text{Expense}$).
    #[must_use]
    pub fn calculate_net_profit(
        entries: &[GlEntry],
        accounts: &HashMap<String, Account>,
        as_of: NaiveDate,
    ) -> Decimal {
        let mut total_income = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;

        for e in entries {
            if e.posting_date <= as_of {
                if let Some(acct) = accounts.get(&e.account) {
                    match acct.root_type {
                        RootType::Income => total_income += e.credit - e.debit,
                        RootType::Expense => total_expense += e.debit - e.credit,
                        _ => {}
                    }
                }
            }
        }

        total_income - total_expense
    }

    /// Verifies the Fundamental Accounting Equation: $\text{Assets} = \text{Liabilities} + \text{Equity} + \text{Net Profit}$.
    #[must_use]
    pub fn verify_balance_sheet(
        entries: &[GlEntry],
        accounts: &HashMap<String, Account>,
        as_of: NaiveDate,
    ) -> bool {
        let mut total_assets = Decimal::ZERO;
        let mut total_liabilities = Decimal::ZERO;
        let mut total_equity = Decimal::ZERO;

        for e in entries {
            if e.posting_date <= as_of {
                if let Some(acct) = accounts.get(&e.account) {
                    match acct.root_type {
                        RootType::Asset => total_assets += e.debit - e.credit,
                        RootType::Liability => total_liabilities += e.credit - e.debit,
                        RootType::Equity => total_equity += e.credit - e.debit,
                        _ => {}
                    }
                }
            }
        }

        let net_profit = Self::calculate_net_profit(entries, accounts, as_of);
        total_assets == total_liabilities + total_equity + net_profit
    }
}
