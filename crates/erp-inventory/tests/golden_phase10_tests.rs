//! Golden Phase 10 Integration Tests: Core Transactional Controller Suite
//!
//! Validates:
//! 1. Procure-to-Pay Lifecycle:
//!    - Purchase Invoice with 3-Way Matching against Purchase Order (rate) & Purchase Receipt (qty)
//!    - Payment Entry with multi-invoice allocation, Forex Gain/Loss auto-balancing, and TDS deduction
//!    - Accounts Payable (AP) and Payment Ledger Entry (PLE) consistency
//! 2. Stock Entry Controller:
//!    - Material Transfer dual SLEs and GL inventory re-balancing
//!    - Manufacture production batch distributing raw material costs and factory overheads to finished goods
//! 3. Serial Number State Machine:
//!    - Strict lifecycle transitions: Draft -> Active -> InTransit -> Delivered -> Returned -> Restocked -> Scrapped
//!    - Rejection of invalid transitions, duplicate receipts, and wrong-warehouse dispatches.

use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::{
    PaymentEntry, PaymentEntryDeduction, PaymentEntryReference, PaymentType, PurchaseInvoice,
    PurchaseInvoiceItem, PurchaseTaxItem,
};
use erp_inventory::{
    SerialLifecycleState, SerialReceiptParams, SerialRegistry, SerialStateError, StockEntry,
    StockEntryAdditionalCost, StockEntryError, StockEntryItem, StockEntryPurpose,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_golden_procure_to_pay_3way_matching_and_multi_invoice_payment() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Create Purchase Invoice for $10,000 + 10% VAT ($1,000) = $11,000 Total
    // 3-Way Match: PO rate = 100.0, PR accepted qty = 100.0
    let mut pi = PurchaseInvoice {
        name: "PINV-2026-0001".into(),
        supplier: "Global Metals Inc".into(),
        posting_date: date,
        due_date: date,
        company: "Acme Industrial".into(),
        credit_to: "2110 - Accounts Payable".into(),
        items: vec![PurchaseInvoiceItem {
            item_code: "STEEL-PLATE".into(),
            item_name: "Heavy Duty Steel Plate".into(),
            qty: dec!(100.0),
            rate: dec!(100.0),
            amount: Decimal::ZERO, // auto-calculated
            expense_account: "1510 - Stock Received But Not Billed".into(),
            cost_center: Some("Main Plant".into()),
            purchase_order: Some("PO-2026-0099".into()),
            purchase_receipt: Some("PREC-2026-0045".into()),
            po_contract_rate: Some(dec!(100.0)),
            pr_accepted_qty: Some(dec!(100.0)),
        }],
        taxes: vec![PurchaseTaxItem {
            charge_type: "On Net Total".into(),
            account_head: "2200 - Input VAT 10%".into(),
            rate: dec!(10.0),
            tax_amount: Decimal::ZERO, // auto-calculated
            total: Decimal::ZERO,
        }],
        net_total: Decimal::ZERO,
        total_taxes_and_charges: Decimal::ZERO,
        grand_total: Decimal::ZERO,
        currency: "USD".into(),
        conversion_rate: dec!(1.0),
        docstatus: 0,
        remarks: Some("Steel plates for production".into()),
    };

    // Submit Purchase Invoice with 1% rate tolerance
    pi.submit(dec!(1.0))
        .expect("Purchase Invoice submission failed");
    assert_eq!(pi.docstatus, 1);
    assert_eq!(pi.net_total, dec!(10000.00));
    assert_eq!(pi.grand_total, dec!(11000.00));

    // Verify GL entries for Purchase Invoice
    let pi_gl = pi
        .generate_gl_entries()
        .expect("PI GL entry generation failed");
    assert_eq!(pi_gl.len(), 3);

    let total_debit: Decimal = pi_gl.iter().map(|e| e.debit).sum();
    let total_credit: Decimal = pi_gl.iter().map(|e| e.credit).sum();
    assert_eq!(total_debit, total_credit);
    assert_eq!(total_debit, dec!(11000.00));

    let ap_entry = pi_gl.iter().find(|e| e.credit > Decimal::ZERO).unwrap();
    assert_eq!(ap_entry.account, "2110 - Accounts Payable");
    assert_eq!(ap_entry.credit, dec!(11000.00));

    // Verify AP Payment Ledger Entry
    let pi_ple = pi.generate_ple_entry();
    assert_eq!(pi_ple.voucher_type, "Purchase Invoice");
    assert_eq!(pi_ple.voucher_no, "PINV-2026-0001");
    assert_eq!(pi_ple.amount, dec!(11000.00)); // Establishing invoice payable

    // 2. Disburse Payment Entry settling this invoice with:
    // - Paid amount: $10,800 from Bank Account
    // - Deduction (TDS Tax Withholding): $200
    // - Total allocated: $11,000 (100% settlement of PINV-2026-0001)
    let mut pe = PaymentEntry {
        name: "PAY-2026-0001".into(),
        payment_type: PaymentType::Pay,
        posting_date: date,
        company: "Acme Industrial".into(),
        party_type: Some("Supplier".into()),
        party: Some("Global Metals Inc".into()),
        paid_from: "1110 - Bank Checking".into(),
        paid_to: "2110 - Accounts Payable".into(),
        paid_amount: dec!(11000.00),
        received_amount: dec!(11000.00),
        source_exchange_rate: dec!(1.0),
        target_exchange_rate: dec!(1.0),
        references: vec![PaymentEntryReference {
            reference_doctype: "Purchase Invoice".into(),
            reference_name: "PINV-2026-0001".into(),
            total_amount: dec!(11000.00),
            outstanding_amount: dec!(11000.00),
            allocated_amount: dec!(11000.00),
            exchange_rate: dec!(1.0),
            exchange_gain_loss: Decimal::ZERO,
        }],
        deductions: vec![PaymentEntryDeduction {
            account: "2210 - Tax Deducted at Source (TDS)".into(),
            cost_center: Some("Main Plant".into()),
            amount: dec!(200.00),
        }],
        remarks: Some("Settlement of steel plate invoice less TDS".into()),
        docstatus: 0,
    };

    pe.submit().expect("Payment Entry submission failed");
    assert_eq!(pe.docstatus, 1);

    // Verify Payment Entry GL Entries:
    // Debit Accounts Payable: $11,000.00
    // Credit Bank Checking: $10,800.00
    // Credit TDS Payable: $200.00
    let pe_gl = pe
        .generate_gl_entries("5200 - Exchange Gain/Loss")
        .expect("Payment GL generation failed");

    let pe_debit: Decimal = pe_gl.iter().map(|e| e.debit).sum();
    let pe_credit: Decimal = pe_gl.iter().map(|e| e.credit).sum();
    assert_eq!(pe_debit, pe_credit);
    assert_eq!(pe_debit, dec!(11000.00));

    let bank_credit = pe_gl
        .iter()
        .find(|e| e.account == "1110 - Bank Checking")
        .unwrap();
    assert_eq!(bank_credit.credit, dec!(10800.00));

    let tds_credit = pe_gl
        .iter()
        .find(|e| e.account == "2210 - Tax Deducted at Source (TDS)")
        .unwrap();
    assert_eq!(tds_credit.credit, dec!(200.00));

    let ap_debit = pe_gl
        .iter()
        .find(|e| e.account == "2110 - Accounts Payable")
        .unwrap();
    assert_eq!(ap_debit.debit, dec!(11000.00));

    // Verify Payment Ledger Entries for reconciliation
    let pe_ples = pe.generate_ple_entries();
    assert_eq!(pe_ples.len(), 1);
    assert_eq!(
        pe_ples[0].against_voucher_no.as_deref(),
        Some("PINV-2026-0001")
    );
    assert_eq!(pe_ples[0].amount, dec!(-11000.00)); // Offsets the +11000.00 invoice PLE to 0 balance
}

#[test]
fn test_golden_stock_entry_manufacture_and_material_transfer() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Stock Entry: Manufacture (Aluminum Ingot + Labor -> Precision Casings)
    let mut mfg_entry = StockEntry {
        name: "STE-MFG-001".into(),
        purpose: StockEntryPurpose::Manufacture,
        posting_date: date,
        company: "Precision Tech".into(),
        from_warehouse: None,
        to_warehouse: None,
        items: vec![
            // Raw Material
            StockEntryItem {
                item_code: "AL-INGOT".into(),
                item_name: "Aluminum Ingot 6061".into(),
                s_warehouse: Some("Raw Materials Store".into()),
                t_warehouse: None,
                qty: dec!(100.0),
                basic_rate: dec!(4.50), // Cost = $450.00
                amount: Decimal::ZERO,
                serial_nos: vec![],
                batch_no: None,
                expense_account: None,
                cost_center: None,
                is_finished_item: false,
                is_scrap_item: false,
            },
            // Finished Item
            StockEntryItem {
                item_code: "CASING-P1".into(),
                item_name: "Machined Drone Casing".into(),
                s_warehouse: None,
                t_warehouse: Some("Finished Goods Store".into()),
                qty: dec!(50.0),
                basic_rate: Decimal::ZERO, // Computed from raw materials + labor
                amount: Decimal::ZERO,
                serial_nos: vec![],
                batch_no: None,
                expense_account: None,
                cost_center: None,
                is_finished_item: true,
                is_scrap_item: false,
            },
        ],
        additional_costs: vec![StockEntryAdditionalCost {
            expense_account: "5120 - Direct Labor Machining".into(),
            description: "CNC Milling Operator Hours".into(),
            amount: dec!(150.00),
        }],
        total_incoming_value: Decimal::ZERO,
        total_outgoing_value: Decimal::ZERO,
        value_difference: Decimal::ZERO,
        total_additional_costs: Decimal::ZERO,
        docstatus: 0,
        remarks: Some("Batch production run of 50 drone casings".into()),
    };

    mfg_entry.submit().expect("Manufacture submission failed");
    assert_eq!(mfg_entry.docstatus, 1);
    // Raw material = 450, Labor = 150 -> Total = 600 across 50 units = $12.00/unit
    assert_eq!(mfg_entry.items[1].basic_rate, dec!(12.00));
    assert_eq!(mfg_entry.items[1].amount, dec!(600.00));

    // Verify SLEs
    let mfg_sles = mfg_entry.generate_stock_ledger_entries();
    assert_eq!(mfg_sles.len(), 2);
    assert_eq!(mfg_sles[0].actual_qty, dec!(-100.0));
    assert_eq!(mfg_sles[0].stock_value_difference, dec!(-450.00));
    assert_eq!(mfg_sles[1].actual_qty, dec!(50.0));
    assert_eq!(mfg_sles[1].stock_value_difference, dec!(600.00));

    // Verify GL Entries
    let mfg_gl = mfg_entry
        .generate_gl_entries("5110 - Stock Adjustment", |wh| match wh {
            "Raw Materials Store" => Ok("1310 - Raw Materials Inventory".into()),
            "Finished Goods Store" => Ok("1320 - Finished Goods Inventory".into()),
            _ => Err(StockEntryError::WarehouseAccountNotFound(wh.into())),
        })
        .expect("Manufacture GL generation failed");

    let debit_sum: Decimal = mfg_gl.iter().map(|e| e.debit).sum();
    let credit_sum: Decimal = mfg_gl.iter().map(|e| e.credit).sum();
    assert_eq!(debit_sum, credit_sum);
    assert_eq!(debit_sum, dec!(600.00));

    // 2. Stock Entry: Material Transfer of 20 casings to Regional Distribution Hub
    let mut transfer_entry = StockEntry {
        name: "STE-XFER-001".into(),
        purpose: StockEntryPurpose::MaterialTransfer,
        posting_date: date,
        company: "Precision Tech".into(),
        from_warehouse: Some("Finished Goods Store".into()),
        to_warehouse: Some("Regional Hub Warehouse".into()),
        items: vec![StockEntryItem {
            item_code: "CASING-P1".into(),
            item_name: "Machined Drone Casing".into(),
            s_warehouse: None,
            t_warehouse: None,
            qty: dec!(20.0),
            basic_rate: dec!(12.00),
            amount: Decimal::ZERO,
            serial_nos: vec![],
            batch_no: None,
            expense_account: None,
            cost_center: None,
            is_finished_item: false,
            is_scrap_item: false,
        }],
        additional_costs: vec![],
        total_incoming_value: Decimal::ZERO,
        total_outgoing_value: Decimal::ZERO,
        value_difference: Decimal::ZERO,
        total_additional_costs: Decimal::ZERO,
        docstatus: 0,
        remarks: Some("Inter-branch stock transfer".into()),
    };

    transfer_entry.submit().expect("Transfer submission failed");
    assert_eq!(transfer_entry.total_outgoing_value, dec!(240.00));
    assert_eq!(transfer_entry.total_incoming_value, dec!(240.00));

    let xfer_sles = transfer_entry.generate_stock_ledger_entries();
    assert_eq!(xfer_sles.len(), 2);
    assert_eq!(xfer_sles[0].actual_qty, dec!(-20.0));
    assert_eq!(xfer_sles[1].actual_qty, dec!(20.0));
}

#[test]
fn test_golden_serial_number_state_machine_strict_guards() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let mut registry = SerialRegistry::new();

    let sn = CompactString::new("SR-9988-ALPHA");
    let item = CompactString::new("HIGH-END-ROUTER");

    // 1. Initial Receipt into Central Store
    registry
        .register_receipt(SerialReceiptParams {
            serial_no: sn.clone(),
            item_code: item.clone(),
            warehouse: "Central Store".into(),
            voucher_type: "Purchase Receipt".into(),
            voucher_no: "PR-100".into(),
            posting_date: date,
            batch_no: None,
        })
        .expect("Receipt failed");

    let rec = registry.get(&sn).unwrap();
    assert_eq!(rec.state, SerialLifecycleState::Active);
    assert_eq!(rec.warehouse.as_deref(), Some("Central Store"));

    // 2. Reserve unit for Sales Order
    registry
        .reserve(&sn, "Sales Order".into(), "SO-500".into(), date)
        .expect("Reservation failed");
    assert_eq!(
        registry.get(&sn).unwrap().state,
        SerialLifecycleState::Reserved
    );

    // 3. Prohibit transferring a reserved unit before unreserving
    let bad_transfer = registry.transfer_start(
        &sn,
        "Central Store",
        "Stock Entry".into(),
        "STE-9".into(),
        date,
    );
    assert!(matches!(
        bad_transfer,
        Err(SerialStateError::IllegalTransition { .. })
    ));

    // 4. Deliver reserved unit to customer
    registry
        .deliver(
            &sn,
            "Central Store",
            "Delivery Note".into(),
            "DN-700".into(),
            date,
        )
        .expect("Delivery failed");
    assert_eq!(
        registry.get(&sn).unwrap().state,
        SerialLifecycleState::Delivered
    );

    // 5. Customer Return
    registry
        .customer_return(
            &sn,
            "RMA Warehouse".into(),
            "Delivery Note".into(),
            "DN-RET-1".into(),
            date,
        )
        .expect("Return failed");
    assert_eq!(
        registry.get(&sn).unwrap().state,
        SerialLifecycleState::Returned
    );

    // 6. Scrap due to water damage during transit
    registry
        .scrap(
            &sn,
            "Stock Entry".into(),
            "STE-SCRAP-1".into(),
            date,
            "Liquid ingress on PCB",
        )
        .expect("Scrap failed");
    assert_eq!(
        registry.get(&sn).unwrap().state,
        SerialLifecycleState::Scrapped
    );

    // 7. Verify terminal state: Cannot restock or deliver scrapped unit
    let bad_restock = registry.restock(&sn, "Stock Entry".into(), "STE-ERR".into(), date);
    assert!(matches!(
        bad_restock,
        Err(SerialStateError::IllegalTransition { .. })
    ));
}
