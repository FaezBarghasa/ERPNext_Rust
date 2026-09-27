use chrono::NaiveDate;
use erp_accounting::{GlEntry, JournalEntry, JournalEntryLine};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Lending and loan servicing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LendingError {
    /// Zero or invalid loan duration.
    #[error("Loan tenure must be greater than zero")]
    InvalidTenure,
    /// Invalid principal amount.
    #[error("Principal amount must be positive, got {0}")]
    InvalidPrincipal(Decimal),
}

/// An individual installment period in an amortization schedule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AmortizationPeriod {
    /// Period sequence index (1..n).
    pub period_no: usize,
    /// Scheduled installment due date.
    pub payment_date: NaiveDate,
    /// Total installment payment (PMT).
    pub installment_amount: Decimal,
    /// Principal portion of installment.
    pub principal_portion: Decimal,
    /// Interest portion of installment.
    pub interest_portion: Decimal,
    /// Remaining principal balance after this installment.
    pub remaining_balance: Decimal,
}

/// Loan Amortization Engine (Milestone 4.4).
pub struct AmortizationEngine;

impl AmortizationEngine {
    /// Computes the fixed monthly installment PMT using exact fixed-point decimal math.
    /// $$PMT = P \times \frac{r(1+r)^n}{(1+r)^n - 1}$$
    #[must_use]
    pub fn calculate_pmt(principal: Decimal, annual_interest_rate: Decimal, total_periods: usize) -> Decimal {
        if total_periods == 0 || principal <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let r = (annual_interest_rate / dec!(100.0)) / dec!(12.0);
        if r == Decimal::ZERO {
            return (principal / Decimal::from(total_periods)).round_dp(2);
        }

        // Compute (1 + r)^n using Decimal power approximation or f64 bridge for pow
        use rust_decimal::MathematicalOps;
        let one_plus_r = Decimal::ONE + r;
        let factor = one_plus_r.powi(total_periods as i64);

        let pmt = principal * (r * factor) / (factor - Decimal::ONE);
        pmt.round_dp(2)
    }

    /// Generates complete amortization schedule with zero-loss final fractional cent reconciliation.
    pub fn generate_schedule(
        principal: Decimal,
        annual_interest_rate: Decimal,
        total_periods: usize,
        start_date: NaiveDate,
    ) -> Result<Vec<AmortizationPeriod>, LendingError> {
        if total_periods == 0 {
            return Err(LendingError::InvalidTenure);
        }
        if principal <= Decimal::ZERO {
            return Err(LendingError::InvalidPrincipal(principal));
        }

        let pmt = Self::calculate_pmt(principal, annual_interest_rate, total_periods);
        let monthly_rate = (annual_interest_rate / dec!(100.0)) / dec!(12.0);

        let mut schedule = Vec::with_capacity(total_periods);
        let mut current_balance = principal;
        let mut current_date = start_date;

        for period_no in 1..=total_periods {
            // Next month date advance (approximate 30 days)
            current_date = current_date
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap_or(current_date);

            let interest_portion = (current_balance * monthly_rate).round_dp(2);

            let (principal_portion, remaining_balance) = if period_no == total_periods {
                // Final period: pay off exact remaining balance to terminate at exactly 0.00
                (current_balance, Decimal::ZERO)
            } else {
                let p = pmt - interest_portion;
                let rem = (current_balance - p).round_dp(2);
                (p, rem)
            };

            let installment_amount = principal_portion + interest_portion;
            current_balance = remaining_balance;

            schedule.push(AmortizationPeriod {
                period_no,
                payment_date: current_date,
                installment_amount,
                principal_portion,
                interest_portion,
                remaining_balance,
            });
        }

        Ok(schedule)
    }
}

/// Loan Servicing & General Ledger Posting Integration (Milestone 4.5).
pub struct LoanGlEngine;

impl LoanGlEngine {
    /// Generates GL entries for monthly interest accrual:
    /// - Debit: Interest Receivable (Asset increases)
    /// - Credit: Interest Income (Revenue increases)
    #[must_use]
    pub fn create_interest_accrual_gl_entries(
        interest_receivable_account: &str,
        interest_income_account: &str,
        interest_amount: Decimal,
        posting_date: NaiveDate,
        voucher_no: &str,
        company: &str,
    ) -> Vec<GlEntry> {
        vec![
            GlEntry {
                name: format!("{voucher_no}-1"),
                posting_date,
                account: interest_receivable_account.to_string(),
                debit: interest_amount,
                credit: Decimal::ZERO,
                voucher_type: "Loan Interest Accrual".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: None,
                party: None,
                company: company.to_string(),
            },
            GlEntry {
                name: format!("{voucher_no}-2"),
                posting_date,
                account: interest_income_account.to_string(),
                debit: Decimal::ZERO,
                credit: interest_amount,
                voucher_type: "Loan Interest Accrual".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: None,
                party: None,
                company: company.to_string(),
            },
        ]
    }

    /// Generates GL entries for incoming loan repayment installment:
    /// - Debit: Bank Account (Total Installment)
    /// - Credit: Loan Principal Receivable (Principal Portion)
    /// - Credit: Interest Receivable (Interest Portion)
    #[must_use]
    pub fn create_repayment_gl_entries(
        bank_account: &str,
        loan_principal_account: &str,
        interest_receivable_account: &str,
        principal_portion: Decimal,
        interest_portion: Decimal,
        posting_date: NaiveDate,
        voucher_no: &str,
        company: &str,
    ) -> Vec<GlEntry> {
        let total_installment = principal_portion + interest_portion;
        vec![
            GlEntry {
                name: format!("{voucher_no}-1"),
                posting_date,
                account: bank_account.to_string(),
                debit: total_installment,
                credit: Decimal::ZERO,
                voucher_type: "Loan Repayment".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: None,
                party: None,
                company: company.to_string(),
            },
            GlEntry {
                name: format!("{voucher_no}-2"),
                posting_date,
                account: loan_principal_account.to_string(),
                debit: Decimal::ZERO,
                credit: principal_portion,
                voucher_type: "Loan Repayment".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: None,
                party: None,
                company: company.to_string(),
            },
            GlEntry {
                name: format!("{voucher_no}-3"),
                posting_date,
                account: interest_receivable_account.to_string(),
                debit: Decimal::ZERO,
                credit: interest_portion,
                voucher_type: "Loan Repayment".to_string(),
                voucher_no: voucher_no.to_string(),
                party_type: None,
                party: None,
                company: company.to_string(),
            },
        ]
    }

    /// Converts repayment entries into a validated JournalEntry document.
    #[must_use]
    pub fn repayment_to_journal_entry(
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
}
