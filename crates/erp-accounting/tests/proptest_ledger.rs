use chrono::NaiveDate;
use erp_accounting::coa::{Account, RootType};
use erp_accounting::decimal_ledger::{DecLine, straight_line_dec, verify_balanced_dec};
use erp_accounting::ledger::{
    AccountingError, GlEntry, JournalEntry, JournalEntryLine, LedgerPostingEngine,
    StatementGenerator,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

// Strategy for generating valid monetary amounts in cents/satoshis represented as Decimal
prop_compose! {
    fn arb_decimal_amount()(raw in 1i64..10_000_000i64) -> Decimal {
        Decimal::new(raw, 2) // e.g. 100.25
    }
}

// Strategy for generating exchange rates >= 0.01
prop_compose! {
    fn arb_exchange_rate()(raw in 100i64..500_000i64) -> Decimal {
        Decimal::new(raw, 4)
    }
}

fn make_test_account(name: &str, root_type: RootType) -> Account {
    Account {
        name: name.to_string(),
        account_number: None,
        parent_account: None,
        is_group: false,
        root_type,
        account_currency: "USD".to_string(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_balanced_journal_entry_always_validates(
        amounts in prop::collection::vec(arb_decimal_amount(), 1..20),
        rate in arb_exchange_rate(),
    ) {
        let total: Decimal = amounts.iter().copied().sum();

        let mut lines = Vec::new();
        for (i, amt) in amounts.iter().enumerate() {
            lines.push(JournalEntryLine {
                account: format!("Acct-Debit-{i}"),
                debit: *amt,
                credit: Decimal::ZERO,
                debit_in_account_currency: *amt,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: rate,
                party_type: None,
                party: None,
            });
        }
        lines.push(JournalEntryLine {
            account: "Acct-Credit-Balancing".to_string(),
            debit: Decimal::ZERO,
            credit: total,
            credit_in_account_currency: total,
            debit_in_account_currency: Decimal::ZERO,
            exchange_rate: rate,
            party_type: None,
            party: None,
        });

        let entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            company: "PropTest Corp".to_string(),
            lines,
            remarks: "PropTest balanced journal".to_string(),
        };

        prop_assert!(entry.validate_balance().is_ok());

        // Also check DecLine invariant
        let dec_lines: Vec<DecLine> = entry.lines.iter().map(|l| DecLine {
            account: l.account.clone(),
            debit: l.debit,
            credit: l.credit,
        }).collect();
        prop_assert!(verify_balanced_dec(&dec_lines));
    }

    #[test]
    fn prop_unbalanced_journal_entry_strictly_rejected(
        debit_amt in arb_decimal_amount(),
        imbalance in 1i64..1_000_000i64,
        rate in arb_exchange_rate(),
    ) {
        let credit_amt = debit_amt + Decimal::new(imbalance, 2);

        let lines = vec![
            JournalEntryLine {
                account: "Acct-Debit".to_string(),
                debit: debit_amt,
                credit: Decimal::ZERO,
                debit_in_account_currency: debit_amt,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: rate,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "Acct-Credit".to_string(),
                debit: Decimal::ZERO,
                credit: credit_amt,
                credit_in_account_currency: credit_amt,
                debit_in_account_currency: Decimal::ZERO,
                exchange_rate: rate,
                party_type: None,
                party: None,
            },
        ];

        let entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            company: "PropTest Corp".to_string(),
            lines,
            remarks: "Unbalanced".to_string(),
        };

        match entry.validate_balance() {
            Err(AccountingError::UnbalancedTransaction { total_debit, total_credit }) => {
                prop_assert_eq!(total_debit, debit_amt);
                prop_assert_eq!(total_credit, credit_amt);
            }
            other => prop_assert!(false, "Expected UnbalancedTransaction, got {:?}", other),
        }
    }

    #[test]
    fn prop_balance_exchange_variance_restores_equilibrium(
        debit in arb_decimal_amount(),
        credit in arb_decimal_amount(),
    ) {
        let mut entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            company: "PropTest Corp".to_string(),
            lines: vec![
                JournalEntryLine {
                    account: "Cash-USD".to_string(),
                    debit,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: debit,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "AP-Vendor".to_string(),
                    debit: Decimal::ZERO,
                    credit,
                    credit_in_account_currency: credit,
                    debit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Multi-currency variance test".to_string(),
        };

        prop_assert!(entry.balance_exchange_variance("Exchange Gain or Loss").is_ok());
        prop_assert!(entry.validate_balance().is_ok());

        let total_debit: Decimal = entry.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = entry.lines.iter().map(|l| l.credit).sum();
        prop_assert_eq!(total_debit, total_credit);
    }

    #[test]
    fn prop_trial_balance_zero_sum_invariant(
        batch_sizes in prop::collection::vec(1usize..5usize, 1..10),
    ) {
        let accounts = vec![
            make_test_account("Bank", RootType::Asset),
            make_test_account("Sales", RootType::Income),
            make_test_account("Expense", RootType::Expense),
            make_test_account("AP", RootType::Liability),
            make_test_account("Capital", RootType::Equity),
        ];
        let engine = LedgerPostingEngine::new(accounts, vec![]);
        let mut all_gl_entries: Vec<GlEntry> = Vec::new();

        for (v_idx, &size) in batch_sizes.iter().enumerate() {
            let leg_amount = Decimal::new((v_idx as i64 + 1) * 100, 2);
            let total = leg_amount * Decimal::from(size);

            let mut lines = Vec::new();
            for i in 0..size {
                let acct = match i % 3 {
                    0 => "Bank",
                    1 => "Expense",
                    _ => "Capital",
                };
                lines.push(JournalEntryLine {
                    account: acct.to_string(),
                    debit: leg_amount,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: leg_amount,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                });
            }
            lines.push(JournalEntryLine {
                account: "Sales".to_string(),
                debit: Decimal::ZERO,
                credit: total,
                credit_in_account_currency: total,
                debit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            });

            let entry = JournalEntry {
                posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                company: "ZeroSum Corp".to_string(),
                lines,
                remarks: "Batch post".to_string(),
            };

            let posted = engine.post_journal_entry(&entry, &format!("JV-{v_idx}")).unwrap();
            all_gl_entries.extend(posted);
        }

        let trial_balance = StatementGenerator::generate_trial_balance(
            &all_gl_entries,
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );

        let sum_of_balances: Decimal = trial_balance.iter().map(|r| r.balance).sum();
        prop_assert_eq!(sum_of_balances, Decimal::ZERO);

        let sum_total_debit: Decimal = trial_balance.iter().map(|r| r.total_debit).sum();
        let sum_total_credit: Decimal = trial_balance.iter().map(|r| r.total_credit).sum();
        prop_assert_eq!(sum_total_debit, sum_total_credit);
    }

    #[test]
    fn prop_balance_sheet_accounting_equation_holds(
        asset_amt in arb_decimal_amount(),
        expense_amt in arb_decimal_amount(),
        income_amt in arb_decimal_amount(),
    ) {
        let mut accounts_map = HashMap::new();
        accounts_map.insert("Bank".to_string(), make_test_account("Bank", RootType::Asset));
        accounts_map.insert("AP".to_string(), make_test_account("AP", RootType::Liability));
        accounts_map.insert("Capital".to_string(), make_test_account("Capital", RootType::Equity));
        accounts_map.insert("Sales".to_string(), make_test_account("Sales", RootType::Income));
        accounts_map.insert("Rent".to_string(), make_test_account("Rent", RootType::Expense));

        let engine = LedgerPostingEngine::new(accounts_map.values().cloned().collect(), vec![]);

        // 1. Initial capital investment: Bank (Asset) Dr = Capital (Equity) Cr
        let jv1 = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            company: "Equation Corp".to_string(),
            lines: vec![
                JournalEntryLine {
                    account: "Bank".to_string(),
                    debit: asset_amt,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: asset_amt,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "Capital".to_string(),
                    debit: Decimal::ZERO,
                    credit: asset_amt,
                    credit_in_account_currency: asset_amt,
                    debit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Capital".to_string(),
        };

        // 2. Sales: Bank (Asset) Dr = Sales (Income) Cr
        let jv2 = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            company: "Equation Corp".to_string(),
            lines: vec![
                JournalEntryLine {
                    account: "Bank".to_string(),
                    debit: income_amt,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: income_amt,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "Sales".to_string(),
                    debit: Decimal::ZERO,
                    credit: income_amt,
                    credit_in_account_currency: income_amt,
                    debit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Revenue".to_string(),
        };

        // 3. Expense: Rent (Expense) Dr = AP (Liability) Cr
        let jv3 = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
            company: "Equation Corp".to_string(),
            lines: vec![
                JournalEntryLine {
                    account: "Rent".to_string(),
                    debit: expense_amt,
                    credit: Decimal::ZERO,
                    debit_in_account_currency: expense_amt,
                    credit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "AP".to_string(),
                    debit: Decimal::ZERO,
                    credit: expense_amt,
                    credit_in_account_currency: expense_amt,
                    debit_in_account_currency: Decimal::ZERO,
                    exchange_rate: Decimal::ONE,
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Expense".to_string(),
        };

        let mut entries = Vec::new();
        entries.extend(engine.post_journal_entry(&jv1, "JV-01").unwrap());
        entries.extend(engine.post_journal_entry(&jv2, "JV-02").unwrap());
        entries.extend(engine.post_journal_entry(&jv3, "JV-03").unwrap());

        let as_of = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let equation_holds = StatementGenerator::verify_balance_sheet(&entries, &accounts_map, as_of);
        prop_assert!(equation_holds, "Balance sheet equation Assets = Liabilities + Equity + Net Profit failed");
    }

    #[test]
    fn prop_straight_line_depreciation_exact_sum(
        cost_raw in 100i64..10_000_000i64,
        salvage_raw in 0i64..99i64,
        periods in 1usize..120usize,
    ) {
        let cost = Decimal::new(cost_raw, 2);
        let salvage = Decimal::new(salvage_raw, 2);
        let schedule = straight_line_dec(cost, salvage, periods);

        prop_assert_eq!(schedule.len(), periods);
        let sum: Decimal = schedule.iter().copied().sum();
        let expected = cost - salvage;
        prop_assert_eq!(sum, expected, "Depreciation schedule sum {} != expected {}", sum, expected);
    }
}
