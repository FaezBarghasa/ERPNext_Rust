//! Automated Zero-Balance Account (ZBA) Cash Pooling & Intercompany Loans.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankAccount {
    pub account_id: String,
    pub entity_id: String,
    pub target_balance: Decimal, // Typically 0.00 for ZBA accounts
    pub current_balance: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZbaSweepTransaction {
    pub sweep_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub sweep_amount: Decimal,
    pub intercompany_loan_ref: String,
    pub annual_interest_rate: Decimal,
}

pub struct TreasuryPoolingEngine;

impl TreasuryPoolingEngine {
    /// Evaluates end-of-day subsidiary accounts and sweeps excess cash into parent concentration account.
    pub fn compute_eod_sweeps(
        subsidiary_accounts: &mut [BankAccount],
        parent_account_id: &str,
        intercompany_interest_rate: Decimal,
    ) -> Vec<ZbaSweepTransaction> {
        let mut sweeps = Vec::new();

        for acct in subsidiary_accounts.iter_mut() {
            let excess = acct.current_balance - acct.target_balance;
            if excess > Decimal::ZERO {
                let sweep = ZbaSweepTransaction {
                    sweep_id: format!("SWEEP-{}", acct.account_id),
                    from_account_id: acct.account_id.clone(),
                    to_account_id: parent_account_id.to_string(),
                    sweep_amount: excess,
                    intercompany_loan_ref: format!("IC-LOAN-{}", acct.entity_id),
                    annual_interest_rate: intercompany_interest_rate,
                };
                acct.current_balance = acct.target_balance;
                sweeps.push(sweep);
            }
        }

        sweeps
    }

    /// Computes daily intercompany interest accrual: (Loan Principal * Annual Rate) / 365.
    #[must_use]
    pub fn daily_interest_accrual(principal: Decimal, annual_rate: Decimal) -> Decimal {
        (principal * (annual_rate / Decimal::from(365))).round_dp(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_zba_cash_sweeps_and_interest() {
        let mut subs = vec![BankAccount {
            account_id: "SUB-DE-BANK".into(),
            entity_id: "Sub_Germany".into(),
            target_balance: dec!(10000),  // Reserve target
            current_balance: dec!(65000), // $55k excess
        }];

        let sweeps =
            TreasuryPoolingEngine::compute_eod_sweeps(&mut subs, "PARENT-TREASURY-01", dec!(0.05));
        assert_eq!(sweeps.len(), 1);
        assert_eq!(sweeps[0].sweep_amount, dec!(55000));
        assert_eq!(subs[0].current_balance, dec!(10000));

        let daily_int = TreasuryPoolingEngine::daily_interest_accrual(dec!(55000), dec!(0.05));
        assert!(daily_int > dec!(7.5) && daily_int < dec!(7.6));
    }
}
