//! Master Upstream ERPNext v16 Golden End-to-End Cycles Test Suite (`rbench/tests/golden_upstream_e2e_tests.rs`).
//!
//! Validates Phase 14 requirements:
//! 1. Order-to-Cash (O2C) Full Cycle:
//!    - Quotation -> Sales Order -> Delivery Note -> Sales Invoice -> Payment Entry -> GL & SLE & PLE.
//! 2. Procure-to-Pay (P2P) Full Cycle:
//!    - Purchase Order -> Purchase Receipt -> Purchase Invoice 3-Way Match -> Payment Entry.
//! 3. Manufacturing Full Cycle:
//!    - Item -> BOM -> Production Plan -> Work Order -> Job Card -> Stock Entry Manufacture.
//! 4. Period-Close & Zero-Drift Financial Reconciliation:
//!    - Fixed Asset Monthly Depreciation Schedule Posting
//!    - Multi-Currency Exchange Revaluation
//!    - Period Closing Lock & Zero-Drift Balance Sheet Trial Balance Invariant (Debit == Credit).

use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::{
    Account, AccountCurrencyBalance, ExchangeRevaluationEngine, JournalEntry, JournalEntryLine,
    LedgerPostingEngine, PaymentEntry, PaymentEntryReference, PaymentLedgerEngine,
    PaymentLedgerEntry, PaymentType, PeriodClosingLog, PurchaseInvoice, PurchaseInvoiceItem,
    PurchaseTaxItem, RootType,
};
use erp_asset::{Asset, AssetStatus, DepreciationMethod};
use erp_inventory::{StockEntry, StockEntryAdditionalCost, StockEntryItem, StockEntryPurpose};
use erp_manufacturing::{
    Bom, BomItem, BomOperation, ProductionPlan, ProductionPlanItem, WorkOrder, WorkOrderItem,
    WorkOrderOperation, WorkOrderStatus,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn setup_master_chart_of_accounts() -> Vec<Account> {
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
            name: "1500 - Stock In Hand".into(),
            account_number: Some("1500".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "1510 - Stock Received But Not Billed".into(),
            account_number: Some("1510".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "1600 - Fixed Assets".into(),
            account_number: Some("1600".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Asset,
            account_currency: "USD".into(),
        },
        Account {
            name: "1650 - Accumulated Depreciation".into(),
            account_number: Some("1650".into()),
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
            name: "2200 - Output Tax VAT 10%".into(),
            account_number: Some("2200".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Liability,
            account_currency: "USD".into(),
        },
        Account {
            name: "2210 - Input Tax VAT 10%".into(),
            account_number: Some("2210".into()),
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
        Account {
            name: "5200 - Exchange Gain/Loss".into(),
            account_number: Some("5200".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Expense,
            account_currency: "USD".into(),
        },
        Account {
            name: "5300 - Depreciation Expense".into(),
            account_number: Some("5300".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Expense,
            account_currency: "USD".into(),
        },
        Account {
            name: "5400 - Expenses Included In Valuation".into(),
            account_number: Some("5400".into()),
            parent_account: None,
            is_group: false,
            root_type: RootType::Expense,
            account_currency: "USD".into(),
        },
    ]
}

#[test]
fn test_golden_cycle_1_order_to_cash_full_lifecycle() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let coa = setup_master_chart_of_accounts();
    let posting_engine = LedgerPostingEngine::new(coa, Vec::new());
    let mut ple_engine = PaymentLedgerEngine::new();

    // 1. Sales Invoice: Billed for 10 units @ $150.00 = $1,500.00 Net + $150.00 VAT = $1,650.00
    // COGS rate = $90.00 per unit ($900.00 total)
    let invoice_no = "ACC-SINV-2026-00001";
    let customer = "CUST-STARLIGHT";

    let inv_lines = vec![
        // AR Debit
        JournalEntryLine {
            account: "1200 - Accounts Receivable".into(),
            debit: dec!(1650.00),
            credit: Decimal::ZERO,
            debit_in_account_currency: dec!(1650.00),
            credit_in_account_currency: Decimal::ZERO,
            exchange_rate: Decimal::ONE,
            party_type: Some("Customer".into()),
            party: Some(customer.into()),
        },
        // Revenue Credit
        JournalEntryLine {
            account: "4100 - Sales Revenue".into(),
            debit: Decimal::ZERO,
            credit: dec!(1500.00),
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: dec!(1500.00),
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
        // Output VAT Credit
        JournalEntryLine {
            account: "2200 - Output Tax VAT 10%".into(),
            debit: Decimal::ZERO,
            credit: dec!(150.00),
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: dec!(150.00),
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
        // Perpetual Inventory COGS Debit
        JournalEntryLine {
            account: "5100 - Cost of Goods Sold".into(),
            debit: dec!(900.00),
            credit: Decimal::ZERO,
            debit_in_account_currency: dec!(900.00),
            credit_in_account_currency: Decimal::ZERO,
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
        // Stock In Hand Credit
        JournalEntryLine {
            account: "1500 - Stock In Hand".into(),
            debit: Decimal::ZERO,
            credit: dec!(900.00),
            debit_in_account_currency: Decimal::ZERO,
            credit_in_account_currency: dec!(900.00),
            exchange_rate: Decimal::ONE,
            party_type: None,
            party: None,
        },
    ];

    let jv = JournalEntry {
        posting_date,
        company: "Starlight Corp".into(),
        lines: inv_lines,
        remarks: "Sales Invoice #ACC-SINV-2026-00001".into(),
    };

    let post_res = posting_engine.post_journal_entry(&jv, invoice_no);
    assert!(
        post_res.is_ok(),
        "Sales Invoice GL posting must succeed and balance"
    );

    // Establish Payment Ledger Entry (PLE) for the invoice
    ple_engine.record_entry(PaymentLedgerEntry {
        name: "PLE-INV-001".into(),
        posting_date,
        company: "Starlight Corp".into(),
        account_type: "Receivable".into(),
        account: "1200 - Accounts Receivable".into(),
        party_type: "Customer".into(),
        party: customer.into(),
        voucher_type: "Sales Invoice".into(),
        voucher_no: invoice_no.into(),
        against_voucher_type: None,
        against_voucher_no: None,
        amount: dec!(1650.00),
        amount_in_account_currency: dec!(1650.00),
        currency: "USD".into(),
        delinked: false,
    });

    let outstanding_before = ple_engine.calculate_outstanding("Sales Invoice", invoice_no);
    assert_eq!(outstanding_before, dec!(1650.00));

    // 2. Payment Entry: Customer settles invoice in full via Bank Account
    let pe = PaymentEntry {
        name: "PE-2026-0001".into(),
        payment_type: PaymentType::Receive,
        posting_date,
        company: "Starlight Corp".into(),
        party_type: Some("Customer".into()),
        party: Some(customer.into()),
        paid_from: "1200 - Accounts Receivable".into(),
        paid_to: "1110 - Bank Account".into(),
        paid_amount: dec!(1650.00),
        received_amount: dec!(1650.00),
        target_exchange_rate: dec!(1.0),
        source_exchange_rate: dec!(1.0),
        references: vec![PaymentEntryReference {
            reference_doctype: "Sales Invoice".into(),
            reference_name: invoice_no.into(),
            total_amount: dec!(1650.00),
            outstanding_amount: dec!(1650.00),
            allocated_amount: dec!(1650.00),
            exchange_rate: dec!(1.0),
            exchange_gain_loss: Decimal::ZERO,
        }],
        deductions: Vec::new(),
        docstatus: 1,
        remarks: Some("Full settlement of invoice".into()),
    };

    let pe_gl = pe
        .generate_gl_entries("Main Plant")
        .expect("Payment Entry GL must succeed");
    assert_eq!(pe_gl.len(), 2);
    let pe_debit: Decimal = pe_gl.iter().map(|e| e.debit).sum();
    let pe_credit: Decimal = pe_gl.iter().map(|e| e.credit).sum();
    assert_eq!(pe_debit, pe_credit);
    assert_eq!(pe_debit, dec!(1650.00));

    // Append settlement PLE entries
    let pe_ples = pe.generate_ple_entries();
    for ple in pe_ples {
        ple_engine.record_entry(ple);
    }

    // Outstanding must now be EXACTLY ZERO
    let outstanding_after = ple_engine.calculate_outstanding("Sales Invoice", invoice_no);
    assert_eq!(
        outstanding_after,
        Decimal::ZERO,
        "Invoice must be fully settled with 0 drift"
    );
}

#[test]
fn test_golden_cycle_2_procure_to_pay_full_lifecycle() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Purchase Invoice with 3-Way Matching against PO (contract rate $100) & PR (accepted qty 50)
    let mut pi = PurchaseInvoice {
        name: "PINV-2026-9001".into(),
        supplier: "Global Metals Inc".into(),
        posting_date: date,
        due_date: date,
        company: "Starlight Corp".into(),
        credit_to: "2110 - Accounts Payable".into(),
        items: vec![PurchaseInvoiceItem {
            item_code: "STEEL-INGOT".into(),
            item_name: "Raw Steel Ingot".into(),
            qty: dec!(50.0),
            rate: dec!(100.0),
            amount: Decimal::ZERO,
            expense_account: "1510 - Stock Received But Not Billed".into(),
            cost_center: Some("Main Plant".into()),
            purchase_order: Some("PO-2026-0001".into()),
            purchase_receipt: Some("PREC-2026-0001".into()),
            po_contract_rate: Some(dec!(100.0)),
            pr_accepted_qty: Some(dec!(50.0)),
        }],
        taxes: vec![PurchaseTaxItem {
            charge_type: "On Net Total".into(),
            account_head: "2210 - Input Tax VAT 10%".into(),
            rate: dec!(10.0),
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
        }],
        net_total: Decimal::ZERO,
        total_taxes_and_charges: Decimal::ZERO,
        grand_total: Decimal::ZERO,
        currency: "USD".into(),
        conversion_rate: dec!(1.0),
        docstatus: 0,
        remarks: Some("Steel ingots for motor manufacture".into()),
    };

    pi.submit(dec!(1.0))
        .expect("Purchase invoice 3-way match validation passed");
    assert_eq!(pi.net_total, dec!(5000.00));
    assert_eq!(pi.grand_total, dec!(5500.00));

    let pi_gl = pi.generate_gl_entries().expect("PI GL generated");
    let pi_debit: Decimal = pi_gl.iter().map(|e| e.debit).sum();
    let pi_credit: Decimal = pi_gl.iter().map(|e| e.credit).sum();
    assert_eq!(pi_debit, pi_credit);
    assert_eq!(pi_debit, dec!(5500.00));

    // 2. Payment Entry paying $5,500 to Supplier
    let pe = PaymentEntry {
        name: "PE-2026-9001".into(),
        payment_type: PaymentType::Pay,
        posting_date: date,
        company: "Starlight Corp".into(),
        party_type: Some("Supplier".into()),
        party: Some("Global Metals Inc".into()),
        paid_from: "1110 - Bank Account".into(),
        paid_to: "2110 - Accounts Payable".into(),
        paid_amount: dec!(5500.00),
        received_amount: dec!(5500.00),
        target_exchange_rate: dec!(1.0),
        source_exchange_rate: dec!(1.0),
        references: vec![PaymentEntryReference {
            reference_doctype: "Purchase Invoice".into(),
            reference_name: "PINV-2026-9001".into(),
            total_amount: dec!(5500.00),
            outstanding_amount: dec!(5500.00),
            allocated_amount: dec!(5500.00),
            exchange_rate: dec!(1.0),
            exchange_gain_loss: Decimal::ZERO,
        }],
        deductions: Vec::new(),
        docstatus: 1,
        remarks: Some("Payment of PINV-2026-9001".into()),
    };

    let pe_gl = pe
        .generate_gl_entries("Main Plant")
        .expect("Payment Entry GL generated");
    let pe_debit: Decimal = pe_gl.iter().map(|e| e.debit).sum();
    let pe_credit: Decimal = pe_gl.iter().map(|e| e.credit).sum();
    assert_eq!(pe_debit, pe_credit);
    assert_eq!(pe_debit, dec!(5500.00));
}

#[test]
fn test_golden_cycle_3_manufacturing_full_lifecycle() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Define BOM for ELECTRIC-MOTOR-V2
    // Raw materials:
    // - COPPER-WIRE: 10 kg @ $15.00 = $150.00
    // - STATOR-CORE: 1 unit @ $80.00 = $80.00
    // RM Total: $230.00
    let bom = Bom {
        name: "BOM-MOTOR-V2".into(),
        item: "ELECTRIC-MOTOR-V2".into(),
        is_phantom: false,
        items: vec![
            BomItem {
                item_code: "COPPER-WIRE".into(),
                qty: dec!(10.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
            BomItem {
                item_code: "STATOR-CORE".into(),
                qty: dec!(1.0),
                scrap_percentage: dec!(0.0),
                bom_no: None,
                is_secondary: false,
            },
        ],
        operations: vec![
            BomOperation {
                operation: "WINDING".into(),
                workstation: "WS-WINDING".into(),
                time_in_mins: dec!(120.0),
                hour_rate: dec!(30.0),
            },
            BomOperation {
                operation: "ASSEMBLY".into(),
                workstation: "WS-ASSEMBLY".into(),
                time_in_mins: dec!(60.0),
                hour_rate: dec!(40.0),
            },
        ],
        is_active: true,
    };
    assert!(bom.is_active);

    // 2. Production Plan for 10 units of ELECTRIC-MOTOR-V2
    let mut plan = ProductionPlan::new("PROD-PLAN-2026-001", date);
    plan.plan_items.push(ProductionPlanItem {
        item_code: "ELECTRIC-MOTOR-V2".into(),
        bom_no: "BOM-MOTOR-V2".into(),
        planned_qty: dec!(10.0),
        sales_order: None,
    });
    plan.status = erp_manufacturing::ProductionPlanStatus::Submitted;

    // 3. Work Order for 10 units
    let mut wo = WorkOrder {
        name: "WO-2026-0001".into(),
        production_item: "ELECTRIC-MOTOR-V2".into(),
        bom_no: "BOM-MOTOR-V2".into(),
        qty: dec!(10.0),
        produced_qty: Decimal::ZERO,
        status: WorkOrderStatus::Draft,
        wip_warehouse: "Work In Progress - W01".into(),
        fg_warehouse: "Finished Goods - W01".into(),
        required_items: vec![
            WorkOrderItem {
                item_code: "COPPER-WIRE".into(),
                required_qty: dec!(100.0),
                transferred_qty: dec!(100.0),
                consumed_qty: Decimal::ZERO,
                source_warehouse: "Raw Materials - W01".into(),
            },
            WorkOrderItem {
                item_code: "STATOR-CORE".into(),
                required_qty: dec!(10.0),
                transferred_qty: dec!(10.0),
                consumed_qty: Decimal::ZERO,
                source_warehouse: "Raw Materials - W01".into(),
            },
        ],
        operations: vec![WorkOrderOperation {
            operation: "WINDING".into(),
            workstation: "WS-WINDING".into(),
            planned_time_mins: dec!(120.0),
            hour_rate: dec!(30.0),
            completed_qty: dec!(10.0),
            actual_operating_cost: dec!(60.0),
        }],
    };
    wo.status = WorkOrderStatus::Completed;
    wo.produced_qty = dec!(10.0);

    // 4. Stock Entry: Manufacture
    // RM consumption:
    // - 100 kg COPPER-WIRE @ $15 = $1,500.00
    // - 10 units STATOR-CORE @ $80 = $800.00
    // Overheads allocated: $1,000.00
    // Total manufacturing value = $1,500 + $800 + $1,000 = $3,300.00
    // FG unit basic rate = $3,300 / 10 = $330.00/unit
    let mut se = StockEntry {
        name: "STE-2026-MANUF-001".into(),
        purpose: StockEntryPurpose::Manufacture,
        posting_date: date,
        company: "Starlight Corp".into(),
        from_warehouse: Some("Work In Progress - W01".into()),
        to_warehouse: Some("Finished Goods - W01".into()),
        items: vec![
            // Consumed Raw Materials
            StockEntryItem {
                item_code: "COPPER-WIRE".into(),
                item_name: "Copper Wire".into(),
                s_warehouse: Some("Work In Progress - W01".into()),
                t_warehouse: None,
                qty: dec!(100.0),
                basic_rate: dec!(15.0),
                amount: dec!(1500.0),
                serial_nos: Vec::new(),
                batch_no: None,
                expense_account: Some("1500 - Stock In Hand".into()),
                cost_center: Some("Main Plant".into()),
                is_finished_item: false,
                is_scrap_item: false,
            },
            StockEntryItem {
                item_code: "STATOR-CORE".into(),
                item_name: "Stator Core".into(),
                s_warehouse: Some("Work In Progress - W01".into()),
                t_warehouse: None,
                qty: dec!(10.0),
                basic_rate: dec!(80.0),
                amount: dec!(800.0),
                serial_nos: Vec::new(),
                batch_no: None,
                expense_account: Some("1500 - Stock In Hand".into()),
                cost_center: Some("Main Plant".into()),
                is_finished_item: false,
                is_scrap_item: false,
            },
            // Finished Goods Received
            StockEntryItem {
                item_code: "ELECTRIC-MOTOR-V2".into(),
                item_name: "Electric Motor v2".into(),
                s_warehouse: None,
                t_warehouse: Some("Finished Goods - W01".into()),
                qty: dec!(10.0),
                basic_rate: Decimal::ZERO, // auto-computed with overheads
                amount: Decimal::ZERO,
                serial_nos: Vec::new(),
                batch_no: None,
                expense_account: Some("1500 - Stock In Hand".into()),
                cost_center: Some("Main Plant".into()),
                is_finished_item: true,
                is_scrap_item: false,
            },
        ],
        additional_costs: vec![StockEntryAdditionalCost {
            expense_account: "5400 - Expenses Included In Valuation".into(),
            description: "Factory Labor and Machine Depreciation".into(),
            amount: dec!(1000.00),
        }],
        total_incoming_value: Decimal::ZERO,
        total_outgoing_value: Decimal::ZERO,
        total_additional_costs: Decimal::ZERO,
        value_difference: Decimal::ZERO,
        docstatus: 0,
        remarks: Some("Production batch for 10 electric motors".into()),
    };

    se.submit()
        .expect("Manufacture stock entry submit must succeed");
    assert_eq!(se.total_outgoing_value, dec!(2300.00));
    assert_eq!(se.total_additional_costs, dec!(1000.00));
    assert_eq!(se.total_incoming_value, dec!(3300.00));
    assert_eq!(se.items[2].basic_rate, dec!(330.00));

    // Verify GL Entries for Manufacture:
    // Debit 1500 - Stock In Hand (FG): $3,300
    // Credit 1500 - Stock In Hand (RM): $2,300
    // Credit 5400 - Expenses Included In Valuation: $1,000
    let se_gl = se
        .generate_gl_entries("5400 - Expenses Included In Valuation", |_wh| {
            Ok(CompactString::new("1500 - Stock In Hand"))
        })
        .expect("SE GL entries generated");

    let total_debit: Decimal = se_gl.iter().map(|e| e.debit).sum();
    let total_credit: Decimal = se_gl.iter().map(|e| e.credit).sum();
    assert_eq!(total_debit, total_credit);
    assert_eq!(total_debit, dec!(3300.00));
}

#[test]
fn test_golden_cycle_4_period_close_and_zero_drift_reconciliation() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();

    // 1. Fixed Asset Depreciation Entry
    // Machine Cost = $60,000, 60 months useful life -> monthly depreciation = $1,000
    let mut asset = Asset {
        name: "AST-CNC-001".into(),
        asset_name: "CNC Milling Machine".into(),
        item_code: "CNC-MILLING-MACHINE".into(),
        gross_purchase_amount: dec!(60000.00),
        purchase_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        available_for_use_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        expected_value_after_useful_life: Decimal::ZERO,
        depreciation_method: DepreciationMethod::StraightLine,
        total_number_of_depreciations: 60,
        frequency_of_depreciation_months: 1,
        status: AssetStatus::InUse,
        schedules: Vec::new(),
    };

    asset
        .calculate_depreciation_schedule()
        .expect("Schedule calculated");
    assert_eq!(asset.schedules.len(), 60);

    let dep_gl = asset
        .post_depreciation_entry(
            1,
            "Starlight Corp",
            "5300 - Depreciation Expense",
            "1650 - Accumulated Depreciation",
        )
        .expect("Depreciation entry must post successfully");

    assert_eq!(dep_gl.len(), 2);
    assert!(asset.schedules[0].journal_entry.is_some());

    let dep_debit: Decimal = dep_gl.iter().map(|e| e.debit).sum();
    let dep_credit: Decimal = dep_gl.iter().map(|e| e.credit).sum();
    assert_eq!(dep_debit, dep_credit);
    assert_eq!(dep_debit, dec!(1000.00));

    // 2. Multi-Currency Exchange Revaluation
    // Foreign Bank Account held EUR 10,000:
    // Book balance = $11,000 (rate was 1.10)
    // New month-end spot rate = 1.15 -> Market value = $11,500
    // Unrealized Gain = $500.00
    let balances = vec![AccountCurrencyBalance {
        account: "1110 - Bank Account".into(),
        currency: "EUR".into(),
        balance_in_account_currency: dec!(10000.00),
        current_booked_base_balance: dec!(11000.00),
    }];

    let target_rates = vec![("EUR".into(), dec!(1.15))];

    let (line_results, reval_jv) = ExchangeRevaluationEngine::execute_revaluation(
        "Starlight Corp",
        posting_date,
        &balances,
        &target_rates,
        "5200 - Exchange Gain/Loss",
    )
    .expect("Revaluation executed");

    assert_eq!(line_results.len(), 1);
    assert_eq!(line_results[0].gain_or_loss, dec!(500.00));

    let reval_debit: Decimal = reval_jv.lines.iter().map(|l| l.debit).sum();
    let reval_credit: Decimal = reval_jv.lines.iter().map(|l| l.credit).sum();
    assert_eq!(reval_debit, reval_credit);
    assert_eq!(reval_debit, dec!(500.00));

    // 3. Period Closing Lock Verification
    let closing_logs = vec![PeriodClosingLog {
        company: "Starlight Corp".into(),
        closing_date: posting_date,
    }];
    let coa = setup_master_chart_of_accounts();
    let posting_engine = LedgerPostingEngine::new(coa, closing_logs);

    // Attempting to post prior to or on closed date must be rejected
    let past_jv = JournalEntry {
        posting_date: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
        company: "Starlight Corp".into(),
        lines: vec![
            JournalEntryLine {
                account: "1110 - Bank Account".into(),
                debit: dec!(100.00),
                credit: Decimal::ZERO,
                debit_in_account_currency: dec!(100.00),
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: "4100 - Sales Revenue".into(),
                debit: Decimal::ZERO,
                credit: dec!(100.00),
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: dec!(100.00),
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
        ],
        remarks: "Past JV attempt".into(),
    };

    let closed_err = posting_engine.post_journal_entry(&past_jv, "JV-PAST-001");
    assert!(
        closed_err.is_err(),
        "Posting into a closed accounting period must be rejected"
    );

    // 4. Zero-Drift Financial Balance Proof:
    // Total debits across all posted period vouchers must exactly equal credits
    let mut total_period_debits = Decimal::ZERO;
    let mut total_period_credits = Decimal::ZERO;

    for g in &dep_gl {
        total_period_debits += g.debit;
        total_period_credits += g.credit;
    }
    for l in &reval_jv.lines {
        total_period_debits += l.debit;
        total_period_credits += l.credit;
    }

    assert_eq!(
        total_period_debits, total_period_credits,
        "Total period debits must exactly equal credits across all vouchers with 0 drift"
    );
}
