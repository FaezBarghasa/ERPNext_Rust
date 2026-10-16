//! Multi-Currency Exchange Rate Revaluation Engine (`erp-accounting::exchange_revaluation`).
//!
//! Computes unrealized foreign exchange gains and losses across foreign currency bank,
//! debtor, and creditor balances on fiscal period close dates, generating automated
//! balancing Journal Entries.

use crate::ledger::{AccountingError, JournalEntry, JournalEntryLine};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Snapshot of an account's foreign currency balance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountCurrencyBalance {
    pub account: CompactString,
    pub currency: CompactString,
    pub balance_in_account_currency: Decimal,
    pub current_booked_base_balance: Decimal,
}

/// Result of evaluating an individual account's revaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevaluationLineResult {
    pub account: CompactString,
    pub currency: CompactString,
    pub balance_in_account_currency: Decimal,
    pub current_booked_balance: Decimal,
    pub new_exchange_rate: Decimal,
    pub new_base_balance: Decimal,
    pub gain_or_loss: Decimal, // Positive for gain, negative for loss
}

/// Exchange Rate Revaluation Engine.
pub struct ExchangeRevaluationEngine;

impl ExchangeRevaluationEngine {
    /// Revalues a collection of foreign currency balances against target exchange rates.
    /// Returns revaluation line details and an automated balancing Journal Entry.
    pub fn execute_revaluation(
        company: &str,
        posting_date: NaiveDate,
        balances: &[AccountCurrencyBalance],
        target_rates: &[(CompactString, Decimal)], // (Currency, New Rate to Base)
        unrealized_gain_loss_account: &str,
    ) -> Result<(Vec<RevaluationLineResult>, JournalEntry), AccountingError> {
        let mut line_results = Vec::new();
        let mut journal_lines = Vec::new();
        let mut total_variance = Decimal::ZERO;

        for bal in balances {
            let Some(&(_, new_rate)) = target_rates.iter().find(|(c, _)| c == &bal.currency) else {
                continue;
            };

            if new_rate <= Decimal::ZERO {
                return Err(AccountingError::InvalidExchangeRate(new_rate));
            }

            let new_base_balance = bal.balance_in_account_currency * new_rate;
            let variance = new_base_balance - bal.current_booked_base_balance;

            if variance.is_zero() {
                continue;
            }

            line_results.push(RevaluationLineResult {
                account: bal.account.clone(),
                currency: bal.currency.clone(),
                balance_in_account_currency: bal.balance_in_account_currency,
                current_booked_balance: bal.current_booked_base_balance,
                new_exchange_rate: new_rate,
                new_base_balance,
                gain_or_loss: variance,
            });

            // If variance > 0: Asset value increased (Debit account, Credit Gain/Loss)
            // If variance < 0: Asset value decreased (Credit account, Debit Gain/Loss)
            let (debit, credit) = if variance > Decimal::ZERO {
                (variance, Decimal::ZERO)
            } else {
                (Decimal::ZERO, -variance)
            };

            journal_lines.push(JournalEntryLine {
                account: bal.account.to_string(),
                debit,
                credit,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            });

            total_variance += variance;
        }

        // Add balancing line to Unrealized Exchange Gain/Loss account
        if !total_variance.is_zero() {
            let (gain_debit, gain_credit) = if total_variance > Decimal::ZERO {
                (Decimal::ZERO, total_variance) // Credit Income
            } else {
                (-total_variance, Decimal::ZERO) // Debit Expense
            };

            journal_lines.push(JournalEntryLine {
                account: unrealized_gain_loss_account.to_string(),
                debit: gain_debit,
                credit: gain_credit,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            });
        }

        let journal = JournalEntry {
            posting_date,
            company: company.to_string(),
            lines: journal_lines,
            remarks: format!("Exchange Rate Revaluation on {posting_date}"),
        };

        // Strictly validate balance
        journal.validate_balance()?;

        Ok((line_results, journal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_exchange_rate_revaluation_balancing() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        // Foreign bank account with 10,000 EUR booked at 1.05 = $10,500
        // New exchange rate is 1.10 = $11,000 -> Unrealized Gain of $500
        let balances = vec![AccountCurrencyBalance {
            account: "1120 - EUR Bank Account".into(),
            currency: "EUR".into(),
            balance_in_account_currency: dec!(10000.00),
            current_booked_base_balance: dec!(10500.00),
        }];

        let rates = vec![("EUR".into(), dec!(1.10))];

        let (details, journal) = ExchangeRevaluationEngine::execute_revaluation(
            "Acme Global",
            date,
            &balances,
            &rates,
            "5200 - Exchange Gain/Loss",
        )
        .expect("Revaluation must balance");

        assert_eq!(details.len(), 1);
        assert_eq!(details[0].gain_or_loss, dec!(500.00));

        // Journal Entry must have 2 lines:
        // Debit EUR Bank $500, Credit Exchange Gain/Loss $500
        assert_eq!(journal.lines.len(), 2);
        assert_eq!(journal.lines[0].debit, dec!(500.00));
        assert_eq!(journal.lines[1].credit, dec!(500.00));
        assert!(journal.validate_balance().is_ok());
    }
}
