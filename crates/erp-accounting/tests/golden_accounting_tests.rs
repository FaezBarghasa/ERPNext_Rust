//! Upstream ERPNext Accounting Golden Tests (`erp-accounting`).
//!
//! Validates:
//! 1. Order-to-Cash & Procure-to-Pay General Ledger Posting (`make_gl_entries`).
//! 2. Payment Ledger Entry (PLE) allocation and outstanding tracking.
//! 3. Multi-tier tax engine with previous-row formula resolution and round-off.
//! 4. Multi-currency exchange rate revaluation balancing.
//! 5. AR/AP aging intervals.
//! 6. Property-based zero-drift balance invariant testing with `proptest`.

use chrono::NaiveDate;
use erp_accounting::{
    Account, AccountCurrencyBalance, AgingInvoice, AgingReportEngine, ArApEngine, ChargeType,
    ExchangeRevaluationEngine, JournalEntryLine, LedgerPostingEngine, OpenInvoice,
    PaymentLedgerEngine, PaymentLedgerEntry, PeriodClosingLog, RootType, TaxCalculationEngine,
    TaxCharge,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn sample_coa() -> Vec<Account> {
    vec![
        Account {
            name: "1110 - Bank Account".into(),
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
            name: "2100 - Accounts Payable".into(),
            account_number: Some("2100".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Liability,
            account_currency: "USD".into(),
        },
        Account {
            name: "2200 - Output Tax VAT".into(),
            account_number: Some("2200".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Liability,
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
fn test_golden_sales_invoice_and_payment_gl_posting() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let closed_date = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();

    let closing_logs = vec![PeriodClosingLog {
        company: "Starlight Corp".into(),
        closing_date: closed_date,
    }];

    let engine = LedgerPostingEngine::new(sample_coa(), closing_logs);

    // 1. Sales Invoice Posting:
    // Debit Accounts Receivable: $1,190.00
    // Credit Sales Revenue: $1,000.00
    // Credit Output Tax: $190.00
    let invoice_lines = vec![
        JournalEntryLine {
            account: "1200 - Accounts Receivable".into(),
            debit: dec!(1190.00),
            credit: Decimal::ZERO,
            debit_in_account_currency: dec!(1190.00),
            credit_in_account_currency: Decimal::ZERO,
            exchange_rate: Decimal::ONE,
            party_type: Some("Customer".into()),
            party: Some("CUST-001".into()),
        },
        JournalEntryLine {
            account: "4100 - Sales Revenue".into(),
            debit: Decimal::ZERO,
            credit: dec!(1000.00),
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: dec!(1000.00),
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
        JournalEntryLine {
            account: "2200 - Output Tax VAT".into(),
            debit: Decimal::ZERO,
            credit: dec!(190.00),
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: dec!(190.00),
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
    ];

    let gl_entries = engine
        .make_gl_entries(
            "Sales Invoice",
            "ACC-SINV-2026-00001",
            posting_date,
            "Starlight Corp",
            &invoice_lines,
        )
        .expect("Invoice GL posting must succeed");

    assert_eq!(gl_entries.len(), 3);
    let total_debit: Decimal = gl_entries.iter().map(|e| e.debit).sum();
    let total_credit: Decimal = gl_entries.iter().map(|e| e.credit).sum();
    assert_eq!(total_debit, dec!(1190.00));
    assert_eq!(total_credit, dec!(1190.00));
    assert_eq!(total_debit, total_credit);

    // 2. Closed Period Validation: Posting on 2026-09-15 must fail
    let closed_post_res = engine.make_gl_entries(
        "Sales Invoice",
        "ACC-SINV-OLD",
        NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        "Starlight Corp",
        &invoice_lines,
    );
    assert!(closed_post_res.is_err());
}

#[test]
fn test_golden_payment_ledger_and_reconciliation_flow() {
    let mut ple_engine = PaymentLedgerEngine::new();
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Post Invoice for $2,500
    ple_engine.record_entry(PaymentLedgerEntry {
        name: "PLE-INV-1".into(),
        company: "Starlight Corp".into(),
        posting_date: date,
        account_type: "Receivable".into(),
        party_type: "Customer".into(),
        party: "CUST-001".into(),
        account: "1200 - Accounts Receivable".into(),
        voucher_type: "Sales Invoice".into(),
        voucher_no: "ACC-SINV-001".into(),
        against_voucher_type: None,
        against_voucher_no: None,
        amount: dec!(2500.00),
        amount_in_account_currency: dec!(2500.00),
        currency: "USD".into(),
        delinked: false,
    });

    assert_eq!(
        ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
        dec!(2500.00)
    );

    // 2. Post Advance Payment of $1,500
    ple_engine.record_entry(PaymentLedgerEntry {
        name: "PLE-PAY-1".into(),
        company: "Starlight Corp".into(),
        posting_date: date,
        account_type: "Receivable".into(),
        party_type: "Customer".into(),
        party: "CUST-001".into(),
        account: "1200 - Accounts Receivable".into(),
        voucher_type: "Payment Entry".into(),
        voucher_no: "ACC-PAY-001".into(),
        against_voucher_type: None,
        against_voucher_no: None,
        amount: dec!(-1500.00),
        amount_in_account_currency: dec!(-1500.00),
        currency: "USD".into(),
        delinked: false,
    });

    // 3. Reconcile $1,500
    ple_engine
        .reconcile(
            "ACC-PAY-001",
            "Sales Invoice",
            "ACC-SINV-001",
            dec!(1500.00),
            date,
        )
        .unwrap();

    assert_eq!(
        ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
        dec!(1000.00)
    );

    // 4. Over-allocation check (allocating $1,200 when only $1,000 remains)
    assert!(
        ple_engine
            .reconcile(
                "ACC-PAY-001",
                "Sales Invoice",
                "ACC-SINV-001",
                dec!(1200.00),
                date
            )
            .is_err()
    );

    // 5. Unreconcile
    ple_engine.unreconcile("ACC-PAY-001", "ACC-SINV-001");
    assert_eq!(
        ple_engine.calculate_outstanding("Sales Invoice", "ACC-SINV-001"),
        dec!(2500.00)
    );
}

#[test]
fn test_golden_tax_engine_and_round_off() {
    let net = dec!(999.55);
    let charges = vec![
        TaxCharge {
            idx: 1,
            charge_type: ChargeType::OnNetTotal,
            account_head: "2200 - Output Tax VAT".into(),
            description: "VAT 15%".into(),
            rate: dec!(15.0),
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            included_in_print_rate: false,
            row_id: None,
        },
        TaxCharge {
            idx: 2,
            charge_type: ChargeType::Actual,
            account_head: "5120 - Courier".into(),
            description: "Express Courier".into(),
            rate: Decimal::ZERO,
            tax_amount: dec!(15.20),
            total: Decimal::ZERO,
            included_in_print_rate: false,
            row_id: None,
        },
    ];

    let res = TaxCalculationEngine::calculate(net, &charges, true);

    // Net 999.55 + 15% VAT (149.9325) + 15.20 = 1164.6825
    // Rounded: 1165.00, Rounding Adjustment: 1165 - 1164.6825 = 0.3175
    assert_eq!(res.rounded_total, dec!(1165.00));
    assert_eq!(res.rounding_adjustment, dec!(0.3175));
    assert_eq!(res.grand_total + res.rounding_adjustment, res.rounded_total);
}

#[test]
fn test_golden_exchange_rate_revaluation() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    let balances = vec![AccountCurrencyBalance {
        account: "1110 - Bank Account".into(),
        currency: "EUR".into(),
        balance_in_account_currency: dec!(50000.00),
        current_booked_base_balance: dec!(52000.00),
    }];

    // EUR drops from 1.04 to 1.01 -> New Base = 50,500 -> Unrealized Loss of $1,500
    let target_rates = vec![("EUR".into(), dec!(1.01))];

    let (details, journal) = ExchangeRevaluationEngine::execute_revaluation(
        "Starlight Corp",
        date,
        &balances,
        &target_rates,
        "5200 - Exchange Gain/Loss",
    )
    .unwrap();

    assert_eq!(details[0].gain_or_loss, dec!(-1500.00));
    assert_eq!(journal.lines.len(), 2);
    // Credit Bank $1500, Debit Exchange Loss $1500
    assert_eq!(journal.lines[0].credit, dec!(1500.00));
    assert_eq!(journal.lines[1].debit, dec!(1500.00));
    assert!(journal.validate_balance().is_ok());
}

#[test]
fn test_golden_aging_buckets() {
    let cutoff = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    let invoices = vec![
        AgingInvoice {
            voucher_no: "INV-1".into(),
            party_type: "Customer".into(),
            party: "CUST-A".into(),
            party_name: "Customer Alpha".into(),
            posting_date: NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 10, 10).unwrap(), // 6 days -> 0-30
            outstanding_amount: dec!(100.00),
        },
        AgingInvoice {
            voucher_no: "INV-2".into(),
            party_type: "Customer".into(),
            party: "CUST-A".into(),
            party_name: "Customer Alpha".into(),
            posting_date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 8, 10).unwrap(), // 67 days -> 61-90
            outstanding_amount: dec!(200.00),
        },
    ];

    let report = AgingReportEngine::calculate_aging(&invoices, cutoff);
    assert_eq!(report.len(), 1);
    assert_eq!(report[0].bucket.range_0_30, dec!(100.00));
    assert_eq!(report[0].bucket.range_61_90, dec!(200.00));
    assert_eq!(report[0].bucket.total_outstanding, dec!(300.00));
}

// -----------------------------------------------------------------------------
// Proptests: Ledger Sum Zero-Drift Invariant
// -----------------------------------------------------------------------------
proptest! {
    #[test]
    fn proptest_ledger_balance_zero_drift(
        d1 in 0i64..1_000_000_000,
        d2 in 0i64..1_000_000_000,
        c1 in 0i64..1_000_000_000,
    ) {
        let dec_d1 = Decimal::from(d1) / dec!(100);
        let dec_d2 = Decimal::from(d2) / dec!(100);
        let dec_c1 = Decimal::from(c1) / dec!(100);
        let dec_c2 = (dec_d1 + dec_d2) - dec_c1;

        let lines = vec![
            JournalEntryLine {
                account: "1110 - Bank Account".into(),
                debit: dec_d1,
                credit: Decimal::ZERO,
                debit_in_account_currency: dec_d1,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "1200 - Accounts Receivable".into(),
                debit: dec_d2,
                credit: Decimal::ZERO,
                debit_in_account_currency: dec_d2,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "4100 - Sales Revenue".into(),
                debit: Decimal::ZERO,
                credit: dec_c1,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: dec_c1,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "2200 - Output Tax VAT".into(),
                debit: Decimal::ZERO,
                credit: dec_c2,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: dec_c2,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
        ];

        let entry = erp_accounting::JournalEntry {
            posting_date: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
            company: "Starlight Corp".into(),
            lines,
            remarks: "Proptest balanced journal".into(),
        };

        // Equation must hold strictly without any floating-point drift
        prop_assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn proptest_fifo_payment_allocation_never_exceeds(
        inv1 in 1i64..1_000_000,
        inv2 in 1i64..1_000_000,
        pay in 1i64..2_000_000,
    ) {
        let dec_inv1 = Decimal::from(inv1);
        let dec_inv2 = Decimal::from(inv2);
        let dec_pay = Decimal::from(pay);

        let mut invoices = vec![
            OpenInvoice {
                voucher_no: "INV-1".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                grand_total: dec_inv1,
                outstanding_amount: dec_inv1,
            },
            OpenInvoice {
                voucher_no: "INV-2".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                grand_total: dec_inv2,
                outstanding_amount: dec_inv2,
            },
        ];

        let (allocations, remaining) = ArApEngine::allocate_fifo(&mut invoices, dec_pay);

        let total_allocated: Decimal = allocations.iter().map(|a| a.allocated_amount).sum();
        prop_assert!(total_allocated <= dec_pay);
        prop_assert_eq!(total_allocated + remaining, dec_pay);
        prop_assert!(invoices[0].outstanding_amount >= Decimal::ZERO);
        prop_assert!(invoices[1].outstanding_amount >= Decimal::ZERO);
    }
}
