use chrono::NaiveDate;
use erp_accounting::{AccountingError, JournalEntry, JournalEntryLine};
use proptest::prelude::*;
use rust_decimal::Decimal;

#[test]
fn test_simple_balanced_journal_entry() {
    let entry = JournalEntry {
        posting_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        company: "RustNext Enterprise".into(),
        lines: vec![
            JournalEntryLine {
                account: "1110 - Cash".into(),
                debit: Decimal::new(1000, 2), // $10.00
                credit: Decimal::ZERO,
                debit_in_account_currency: Decimal::new(1000, 2),
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "4110 - Sales Revenue".into(),
                debit: Decimal::ZERO,
                credit: Decimal::new(1000, 2), // $10.00
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: Decimal::new(1000, 2),
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
        ],
        remarks: "Cash sale test".into(),
    };

    assert!(entry.validate_balance().is_ok());
}

#[test]
fn test_unbalanced_journal_entry_rejected() {
    let entry = JournalEntry {
        posting_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        company: "RustNext Enterprise".into(),
        lines: vec![
            JournalEntryLine {
                account: "1110 - Cash".into(),
                debit: Decimal::new(1000, 2),
                credit: Decimal::ZERO,
                debit_in_account_currency: Decimal::new(1000, 2),
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "4110 - Sales Revenue".into(),
                debit: Decimal::ZERO,
                credit: Decimal::new(800, 2), // $8.00 != $10.00
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: Decimal::new(800, 2),
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
        ],
        remarks: "Unbalanced test".into(),
    };

    let result = entry.validate_balance();
    assert!(matches!(
        result,
        Err(AccountingError::UnbalancedTransaction { .. })
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_exchange_variance_auto_balances(
        amount_cents in 1i64..1_000_000i64,
        diff_cents in 1i64..500_000i64,
        is_gain in proptest::bool::ANY
    ) {
        let debit = Decimal::new(amount_cents, 2);
        let credit = if is_gain {
            Decimal::new(amount_cents - diff_cents, 2)
        } else {
            Decimal::new(amount_cents + diff_cents, 2)
        };

        let mut entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
            company: "RustNext Enterprise".into(),
            lines: vec![
                JournalEntryLine {
                    account: "1110 - Cash".into(),
                    debit,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: debit,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "2110 - Accounts Payable".into(),
                    debit: Decimal::ZERO,
                    credit,
                    debit_in_account_currency: Decimal::ZERO,
                    credit_in_account_currency: credit,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Variance test".into(),
        };

        // Initially unbalanced
        prop_assert!(entry.validate_balance().is_err());

        // Balance through variance engine
        let res = entry.balance_exchange_variance("5190 - Exchange Gain/Loss");
        prop_assert!(res.is_ok());

        // Now must be strictly balanced
        prop_assert!(entry.validate_balance().is_ok());
    }
}
