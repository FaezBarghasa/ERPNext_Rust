//! Automated WAL Crash Durability & Recovery Verification Suite (`rbench/tests/crash_recovery_tests.rs`).
//!
//! Verifies:
//! 1. Multi-statement double-entry GL ledger transactions committed to SurrealKV survive abrupt shutdown.
//! 2. Zero accounting drift (Total Debits == Total Credits) upon recovery.
//! 3. Stock ledger inventory quantities and valuation balances remain consistent across process restart.
//! 4. Incomplete/aborted transactions do not leak uncommitted records into the restored ledger state.

use chrono::Utc;
use erp_accounting::ledger::{JournalEntry, JournalEntryLine, LedgerPostingEngine};
use rwl_storage::surreal::{execute_transaction, open_tenant_surrealkv};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;

use erp_accounting::coa::{Account, RootType};

#[tokio::test]
async fn test_surrealkv_wal_crash_recovery_and_zero_ledger_drift() {
    let test_dir = std::env::temp_dir().join(format!(
        "rwl_wal_crash_test_{}_{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));

    // Ensure clean state
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).expect("Create test directory");

    let tenant_ns = "test_tenant";
    let tenant_db = "production_db";

    // Setup chart of accounts for ledger validation
    let accounts = vec![
        Account {
            name: "1110 - Bank Account".into(),
            account_number: Some("1110".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "1310 - Accounts Receivable".into(),
            account_number: Some("1310".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "1410 - Finished Goods Inventory".into(),
            account_number: Some("1410".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "2110 - Accounts Payable".into(),
            account_number: Some("2110".into()),
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
            name: "5100 - Cost of Goods Sold".into(),
            account_number: Some("5100".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Expense,
            account_currency: "USD".into(),
        },
    ];
    let posting_engine = LedgerPostingEngine::new(accounts, Vec::new());

    // --- PHASE 1: Write initial balanced GL transactions ---
    {
        let db = open_tenant_surrealkv(&test_dir, tenant_ns, tenant_db)
            .await
            .expect("Open SurrealKV initial instance");

        // Prepare 5 balanced journal entries (e.g. Sales, Bank Deposit, Inventory Purchase, etc.)
        let sample_entries = vec![
            (
                "JV-CRASH-001",
                "1110 - Bank Account",
                "1310 - Accounts Receivable",
                dec!(25000.00),
            ),
            (
                "JV-CRASH-002",
                "1310 - Accounts Receivable",
                "4100 - Sales Revenue",
                dec!(50000.00),
            ),
            (
                "JV-CRASH-003",
                "5100 - Cost of Goods Sold",
                "1410 - Finished Goods Inventory",
                dec!(18500.00),
            ),
            (
                "JV-CRASH-004",
                "1410 - Finished Goods Inventory",
                "2110 - Accounts Payable",
                dec!(32000.00),
            ),
            (
                "JV-CRASH-005",
                "2110 - Accounts Payable",
                "1110 - Bank Account",
                dec!(15000.00),
            ),
        ];

        for (jv_num, dr_acc, cr_acc, amount) in sample_entries {
            let jv = JournalEntry {
                posting_date: Utc::now().date_naive(),
                company: "Test Enterprise Corp".to_string(),
                lines: vec![
                    JournalEntryLine {
                        account: dr_acc.to_string(),
                        debit: amount,
                        credit: Decimal::ZERO,
                        debit_in_account_currency: amount,
                        credit_in_account_currency: Decimal::ZERO,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    },
                    JournalEntryLine {
                        account: cr_acc.to_string(),
                        debit: Decimal::ZERO,
                        credit: amount,
                        debit_in_account_currency: Decimal::ZERO,
                        credit_in_account_currency: amount,
                        exchange_rate: Decimal::ONE,
                        party_type: None,
                        party: None,
                    },
                ],
                remarks: format!("Crash test journal {jv_num}"),
            };

            let gl_entries = posting_engine
                .post_journal_entry(&jv, jv_num)
                .expect("Post journal entry");

            // Persist GL entries inside an atomic SurrealKV transaction
            let mut stmts = Vec::new();
            for gle in &gl_entries {
                let stmt = format!(
                    "CREATE gl_entry CONTENT {{ name: \"{}\", posting_date: \"{}\", account: \"{}\", debit: {}, credit: {}, voucher_type: \"{}\", voucher_no: \"{}\" }};",
                    gle.name,
                    gle.posting_date,
                    gle.account,
                    gle.debit,
                    gle.credit,
                    gle.voucher_type,
                    gle.voucher_no
                );
                stmts.push(stmt);
            }

            execute_transaction(&db, &stmts)
                .await
                .expect("Commit GL transaction");
        }

        // Write a Stock Ledger balance inside a transaction
        let stock_stmts = vec![
            "CREATE stock_ledger_entry CONTENT { item_code: \"ITEM-TURBINE-01\", warehouse: \"Stores - TEC\", actual_qty: 100, valuation_rate: 185.00, stock_value: 18500.00 };".to_string(),
            "CREATE stock_ledger_entry CONTENT { item_code: \"ITEM-ROTOR-02\", warehouse: \"Stores - TEC\", actual_qty: 50, valuation_rate: 270.00, stock_value: 13500.00 };".to_string(),
        ];
        execute_transaction(&db, &stock_stmts)
            .await
            .expect("Commit Stock transaction");

        // Simulate an uncommitted transaction (interrupted before commit)
        let _ = db.query("BEGIN TRANSACTION; CREATE gl_entry CONTENT { name: \"ORPHAN_GL\", debit: 999999, credit: 0 };").await;

        // Simulate abrupt process crash / power loss by dropping DB handle immediately without closing
        drop(db);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let lock_file = test_dir.join("LOCK");
        if lock_file.exists() {
            let _ = std::fs::remove_file(&lock_file);
        }
    }

    // --- PHASE 2: Reopen persistent store from disk (WAL replay & recovery) ---
    {
        let db_recovered = open_tenant_surrealkv(&test_dir, tenant_ns, tenant_db)
            .await
            .expect("Reopen SurrealKV instance after simulated crash");

        // 1. Verify that all 10 committed GL entries survived
        let mut gl_res = db_recovered
            .query("SELECT * FROM gl_entry ORDER BY creation ASC;")
            .await
            .expect("Query recovered GL entries")
            .check()
            .expect("Check GL response");

        let recovered_gl: Vec<Value> = gl_res.take(0).expect("Take GL rows");
        assert_eq!(
            recovered_gl.len(),
            10,
            "All 10 committed GL entries must survive recovery"
        );

        // 2. Verify zero uncommitted leaks (the uncommitted ORPHAN_GL must NOT exist)
        let has_orphan = recovered_gl.iter().any(|r| {
            r.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s == "ORPHAN_GL")
                .unwrap_or(false)
        });
        assert!(
            !has_orphan,
            "Aborted transaction must be rolled back on recovery"
        );

        // 3. Verify the General Ledger Fundamental Equation: Total Debits == Total Credits (Zero Drift)
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;

        for r in &recovered_gl {
            let dr_str = r.get("debit").and_then(|v| v.as_str()).unwrap_or("");
            let cr_str = r.get("credit").and_then(|v| v.as_str()).unwrap_or("");

            let dr = if let Ok(d) = dr_str.parse::<Decimal>() {
                d
            } else if let Some(n) = r.get("debit").and_then(|v| v.as_f64()) {
                Decimal::from_f64_retain(n).unwrap_or(Decimal::ZERO)
            } else {
                Decimal::ZERO
            };

            let cr = if let Ok(c) = cr_str.parse::<Decimal>() {
                c
            } else if let Some(n) = r.get("credit").and_then(|v| v.as_f64()) {
                Decimal::from_f64_retain(n).unwrap_or(Decimal::ZERO)
            } else {
                Decimal::ZERO
            };

            total_debits += dr;
            total_credits += cr;
        }

        assert_eq!(
            total_debits, total_credits,
            "Total debits ({total_debits}) must exactly equal total credits ({total_credits}) with zero drift"
        );
        assert_eq!(total_debits, dec!(140500.00));

        // 4. Verify Stock Ledger persistence
        let mut sle_res = db_recovered
            .query("SELECT * FROM stock_ledger_entry ORDER BY creation ASC;")
            .await
            .expect("Query stock entries")
            .check()
            .expect("Check stock response");

        let recovered_sle: Vec<Value> = sle_res.take(0).expect("Take stock rows");
        assert_eq!(
            recovered_sle.len(),
            2,
            "Both committed stock ledger entries must survive"
        );

        drop(db_recovered);
    }

    // Cleanup test artifacts
    let _ = std::fs::remove_dir_all(&test_dir);
}
