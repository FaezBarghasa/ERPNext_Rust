pub mod assets;
pub mod coa;
pub mod decimal_ledger;
pub mod ledger;
pub mod receivables;

pub use assets::straight_line_depreciation;
pub use coa::{Account, RootType};
pub use decimal_ledger::{straight_line_dec, verify_balanced_dec, DecLine};
pub use ledger::{
    AccountingError, GlEntry, JournalEntry, JournalEntryLine, LedgerPostingEngine,
    PeriodClosingLog, StatementGenerator, TrialBalanceRow,
};
pub use receivables::{ArApEngine, OpenInvoice, PaymentAllocationResult};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn sample_accounts() -> Vec<Account> {
        vec![
            Account {
                name: "1110 - Bank".into(),
                account_number: Some("1110".into()),
                parent_account: None,
                is_group: false,
                root_type: RootType::Asset,
                account_currency: "USD".into(),
            },
            Account {
                name: "1200 - Accounts Receivable".into(),
                account_number: Some("1200".into()),
                parent_account: None,
                is_group: false,
                root_type: RootType::Asset,
                account_currency: "USD".into(),
            },
            Account {
                name: "4100 - Sales Revenue".into(),
                account_number: Some("4100".into()),
                parent_account: None,
                is_group: false,
                root_type: RootType::Income,
                account_currency: "USD".into(),
            },
            Account {
                name: "5100 - Cost of Goods Sold".into(),
                account_number: Some("5100".into()),
                parent_account: None,
                is_group: false,
                root_type: RootType::Expense,
                account_currency: "USD".into(),
            },
            Account {
                name: "5200 - Exchange Gain/Loss".into(),
                account_number: Some("5200".into()),
                parent_account: None,
                is_group: false,
                root_type: RootType::Expense,
                account_currency: "USD".into(),
            },
        ]
    }

    #[test]
    fn test_unbalanced_journal_entry_rejected() {
        let entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 9, 27).unwrap(),
            company: "Acme Corp".into(),
            lines: vec![
                JournalEntryLine {
                    account: "1110 - Bank".into(),
                    debit: dec!(100.000001),
                    credit: dec!(0.00),
                    debit_in_account_currency: dec!(100.000001),
                    credit_in_account_currency: dec!(0.00),
                    exchange_rate: dec!(1.0),
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "4100 - Sales Revenue".into(),
                    debit: dec!(0.00),
                    credit: dec!(100.00),
                    debit_in_account_currency: dec!(0.00),
                    credit_in_account_currency: dec!(100.00),
                    exchange_rate: dec!(1.0),
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Test Imbalance".into(),
        };

        let res = entry.validate_balance();
        assert!(matches!(
            res,
            Err(AccountingError::UnbalancedTransaction { .. })
        ));
    }

    #[test]
    fn test_exchange_variance_auto_balancing() {
        let mut entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 9, 27).unwrap(),
            company: "Acme Corp".into(),
            lines: vec![
                JournalEntryLine {
                    account: "1110 - Bank".into(),
                    debit: dec!(115.00), // Received $115 at rate 1.15
                    credit: dec!(0.00),
                    debit_in_account_currency: dec!(100.00),
                    credit_in_account_currency: dec!(0.00),
                    exchange_rate: dec!(1.15),
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "1200 - Accounts Receivable".into(),
                    debit: dec!(0.00),
                    credit: dec!(110.00), // Booked invoice was $110 at rate 1.10
                    debit_in_account_currency: dec!(0.00),
                    credit_in_account_currency: dec!(100.00),
                    exchange_rate: dec!(1.10),
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Settlement with Exchange Gain".into(),
        };

        // Imbalanced initially ($115 debit vs $110 credit)
        assert!(entry.validate_balance().is_err());

        // Balance with Exchange Gain/Loss account
        entry
            .balance_exchange_variance("5200 - Exchange Gain/Loss")
            .expect("Auto balance failed");

        assert!(entry.validate_balance().is_ok());
        assert_eq!(entry.lines.len(), 3);
        let gain_line = &entry.lines[2];
        assert_eq!(gain_line.credit, dec!(5.00));
        assert_eq!(gain_line.account, "5200 - Exchange Gain/Loss");
    }

    #[test]
    fn test_period_closed_rejection() {
        let accounts = sample_accounts();
        let closing_logs = vec![PeriodClosingLog {
            company: "Acme Corp".into(),
            closing_date: NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
        }];

        let engine = LedgerPostingEngine::new(accounts, closing_logs);

        let closed_entry = JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(), // Inside closed period
            company: "Acme Corp".into(),
            lines: vec![
                JournalEntryLine {
                    account: "1110 - Bank".into(),
                    debit: dec!(50.00),
                    credit: dec!(0.00),
                    debit_in_account_currency: dec!(50.00),
                    credit_in_account_currency: dec!(0.00),
                    exchange_rate: dec!(1.0),
                    party_type: None,
                    party: None,
                },
                JournalEntryLine {
                    account: "4100 - Sales Revenue".into(),
                    debit: dec!(0.00),
                    credit: dec!(50.00),
                    debit_in_account_currency: dec!(0.00),
                    credit_in_account_currency: dec!(50.00),
                    exchange_rate: dec!(1.0),
                    party_type: None,
                    party: None,
                },
            ],
            remarks: "Backdated transaction".into(),
        };

        let res = engine.post_journal_entry(&closed_entry, "JV-001");
        assert!(matches!(res, Err(AccountingError::PeriodClosed(_))));
    }

    #[test]
    fn test_balance_sheet_equation() {
        let accounts = sample_accounts();
        let acct_map: std::collections::HashMap<String, Account> = accounts
            .iter()
            .map(|a| (a.name.clone(), a.clone()))
            .collect();

        let entries = vec![
            // Dr Bank 500, Cr Revenue 500 (Asset 500, Income 500 -> Net Profit 500)
            GlEntry {
                name: "GL-1".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                account: "1110 - Bank".into(),
                debit: dec!(500.00),
                credit: dec!(0.00),
                voucher_type: "Sales Invoice".into(),
                voucher_no: "INV-1".into(),
                party_type: None,
                party: None,
                company: "Acme Corp".into(),
            },
            GlEntry {
                name: "GL-2".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                account: "4100 - Sales Revenue".into(),
                debit: dec!(0.00),
                credit: dec!(500.00),
                voucher_type: "Sales Invoice".into(),
                voucher_no: "INV-1".into(),
                party_type: None,
                party: None,
                company: "Acme Corp".into(),
            },
            // Dr COGS 200, Cr Bank 200 (Expense 200, Asset -200 -> Net Profit 300, Asset 300)
            GlEntry {
                name: "GL-3".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
                account: "5100 - Cost of Goods Sold".into(),
                debit: dec!(200.00),
                credit: dec!(0.00),
                voucher_type: "Delivery Note".into(),
                voucher_no: "DN-1".into(),
                party_type: None,
                party: None,
                company: "Acme Corp".into(),
            },
            GlEntry {
                name: "GL-4".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
                account: "1110 - Bank".into(),
                debit: dec!(0.00),
                credit: dec!(200.00),
                voucher_type: "Delivery Note".into(),
                voucher_no: "DN-1".into(),
                party_type: None,
                party: None,
                company: "Acme Corp".into(),
            },
        ];

        let as_of = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        assert!(StatementGenerator::verify_balance_sheet(
            &entries, &acct_map, as_of
        ));
        let profit = StatementGenerator::calculate_net_profit(&entries, &acct_map, as_of);
        assert_eq!(profit, dec!(300.00));
    }

    #[test]
    fn test_fifo_payment_allocation_and_credit_limit() {
        let mut invoices = vec![
            OpenInvoice {
                voucher_no: "INV-001".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                grand_total: dec!(100.00),
                outstanding_amount: dec!(100.00),
            },
            OpenInvoice {
                voucher_no: "INV-002".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                grand_total: dec!(200.00),
                outstanding_amount: dec!(200.00),
            },
        ];

        // Allocate payment of 150
        let (allocations, unallocated) = ArApEngine::allocate_fifo(&mut invoices, dec!(150.00));
        assert_eq!(unallocated, dec!(0.00));
        assert_eq!(allocations.len(), 2);
        assert_eq!(allocations[0].voucher_no, "INV-001");
        assert_eq!(allocations[0].allocated_amount, dec!(100.00));
        assert_eq!(allocations[0].remaining_outstanding, dec!(0.00));
        assert_eq!(allocations[1].voucher_no, "INV-002");
        assert_eq!(allocations[1].allocated_amount, dec!(50.00));
        assert_eq!(allocations[1].remaining_outstanding, dec!(150.00));

        // Credit limit check
        let credit_ok = ArApEngine::validate_credit_limit(
            dec!(150.00),
            dec!(50.00),
            dec!(300.00),
            dec!(1000.00),
        );
        assert!(credit_ok.is_ok());

        let credit_breach = ArApEngine::validate_credit_limit(
            dec!(800.00),
            dec!(100.00),
            dec!(300.00),
            dec!(1000.00),
        );
        assert!(matches!(
            credit_breach,
            Err(AccountingError::CreditLimitExceeded { .. })
        ));
    }
}
