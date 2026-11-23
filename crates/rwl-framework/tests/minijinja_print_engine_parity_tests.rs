//! Parity integration tests for Phase 7: Dynamic MiniJinja Print Format & ESC/POS Thermal Receipt Engine.
//!
//! Validates:
//! 1. MiniJinja Jinja2 syntax rendering with doc, items, formatting filters.
//! 2. Custom HTML/CSS invoices with vector company logo and inline barcode/QR code SVGs.
//! 3. Currency and date formatting helpers (e.g. `currency('USD')`, `format_date`).
//! 4. 80mm ESC/POS binary receipt compilation with cut and cash drawer kick signals.

use rust_decimal_macros::dec;
use rwl_framework::minijinja_engine::{
    escpos_consts, EscPosCompiler, EscPosItem, EscPosReceipt, MiniJinjaPrintEngine,
};

#[test]
fn test_minijinja_invoice_template_rendering_parity() {
    let engine = MiniJinjaPrintEngine::new();

    let template_source = r#"<!DOCTYPE html>
<html>
<head><title>{{ doc.name }}</title></head>
<body>
    <h1>{{ company.name }} - {{ doc.doctype }}</h1>
    <p>Invoice No: <strong>{{ doc.name }}</strong></p>
    <p>Date: {{ doc.posting_date | format_date }}</p>
    <p>Customer: {{ customer.name }} (Tax ID: {{ customer.tax_id }})</p>
    
    <table border="1">
        <thead>
            <tr><th>Item</th><th>Qty</th><th>Rate</th><th>Amount</th></tr>
        </thead>
        <tbody>
            {% for item in items %}
            <tr>
                <td>{{ item.item_code }}</td>
                <td>{{ item.qty }}</td>
                <td>{{ item.rate | currency(doc.currency) }}</td>
                <td>{{ item.amount | currency(doc.currency) }}</td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
    
    <h3>Total: {{ doc.grand_total | currency(doc.currency) }}</h3>
    
    <div class="barcodes">
        <img src="{{ barcode(doc.name) }}" alt="Barcode" />
        <img src="{{ qrcode(doc.qr_payload) }}" alt="QR Code" />
    </div>
</body>
</html>"#;

    let invoice_data = serde_json::json!({
        "doc": {
            "doctype": "Sales Invoice",
            "name": "ACC-SINV-2026-00042",
            "posting_date": "2026-11-23",
            "currency": "EUR",
            "grand_total": 4200.50,
            "qr_payload": "https://rustnext.io/verify/ACC-SINV-2026-00042"
        },
        "company": {
            "name": "RustNext Enterprise AG",
            "country": "Germany"
        },
        "customer": {
            "name": "Acme Robotics GmbH",
            "tax_id": "DE314159265"
        },
        "items": [
            {
                "item_code": "ROBOT-ARM-6DOF",
                "qty": 2,
                "rate": 2000.00,
                "amount": 4000.00
            },
            {
                "item_code": "SHIPPING-EXPRESS",
                "qty": 1,
                "rate": 200.50,
                "amount": 200.50
            }
        ]
    });

    let rendered = engine
        .render_template_str(template_source, &invoice_data)
        .expect("MiniJinja invoice template rendering should succeed");

    assert!(rendered.contains("RustNext Enterprise AG - Sales Invoice"));
    assert!(rendered.contains("ACC-SINV-2026-00042"));
    assert!(rendered.contains("Acme Robotics GmbH"));
    assert!(rendered.contains("EUR 4000.00"));
    assert!(rendered.contains("EUR 4200.50"));
    // Barcode inline SVG verification
    assert!(rendered.contains("data:image/svg+xml;utf8,<svg"));
    assert!(rendered.contains("ACC-SINV-2026-00042"));
    // QR code inline SVG verification
    assert!(rendered.contains("https://rustnext.io/verify/ACC-SINV-2026-00042"));
}

#[test]
fn test_escpos_thermal_receipt_bytecode_generation() {
    let receipt = EscPosReceipt {
        store_name: "RustNext Superstore".to_string(),
        branch_info: "Branch #01 - Downtown".to_string(),
        invoice_no: "POS-2026-9812".to_string(),
        cashier: "Alice Developer".to_string(),
        date_str: "2026-11-23 14:15".to_string(),
        items: vec![
            EscPosItem {
                name: "Organic Coffee".to_string(),
                qty: dec!(2),
                price: dec!(4.50),
                amount: dec!(9.00),
            },
            EscPosItem {
                name: "Almond Croissant".to_string(),
                qty: dec!(3),
                price: dec!(3.20),
                amount: dec!(9.60),
            },
        ],
        subtotal: dec!(18.60),
        tax: dec!(1.86),
        grand_total: dec!(20.46),
        payment_mode: "Contactless NFC".to_string(),
        footer_message: "Thank you for shopping with us!\nVisit rustnext.io".to_string(),
        kick_drawer: true,
    };

    let bytecode = EscPosCompiler::compile(&receipt);

    // Verify ESC/POS byte sequence
    // 1. Initializer ESC @
    assert_eq!(&bytecode[0..2], &escpos_consts::INIT);

    // 2. Cash drawer kick pin 2 ESC p 0 50 250
    assert_eq!(&bytecode[2..7], &escpos_consts::DRAWER_KICK_PIN2);

    // 3. Contains ASCII payload
    let ascii_str = String::from_utf8_lossy(&bytecode);
    assert!(ascii_str.contains("RustNext Superstore"));
    assert!(ascii_str.contains("Branch #01 - Downtown"));
    assert!(ascii_str.contains("POS-2026-9812"));
    assert!(ascii_str.contains("Organic Coffee"));
    assert!(ascii_str.contains("20.46"));
    assert!(ascii_str.contains("Contactless NFC"));
    assert!(ascii_str.contains("Thank you for shopping with us!"));

    // 4. Paper cut sequence at end GS V 1 (partial cut)
    let len = bytecode.len();
    assert_eq!(&bytecode[len - 3..len], &escpos_consts::CUT_PAPER_PARTIAL);
}
