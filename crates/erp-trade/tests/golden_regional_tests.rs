//! Upstream ERPNext Regional Tax & E-Invoicing Golden Tests (`erp-trade`).
//!
//! Validates statutory e-invoicing protocols:
//! 1. Peppol BIS 3.0 UBL XML format (European Union).
//! 2. ZATCA Phase 2 cryptographic hash chaining & TLV QR encoding (Saudi Arabia).
//! 3. Moadian (سامانه مودیان) 22-char Tax ID & canonical payload hashing (Iran).

use chrono::{DateTime, NaiveDate, Utc};
use erp_trade::{
    EInvoiceDocument, EInvoiceGenerator, EInvoiceStandard, MoadianInvoicePayload,
    MoadianInvoiceStatus, MoadianItemLine, MoadianTaxIdGenerator, ZatcaPhase2Engine,
};
use rust_decimal_macros::dec;

#[test]
fn test_golden_peppol_bis_3_ubl_generation() {
    let doc = EInvoiceDocument {
        invoice_id: "INV-2026-EU-9821".into(),
        standard: EInvoiceStandard::PeppolBis3,
        seller_tax_id: "DE123456789".into(),
        buyer_tax_id: "FR987654321".into(),
        issue_date: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
        total_taxable_amount: dec!(15000.00),
        total_vat_amount: dec!(2850.00),
        payable_amount: dec!(17850.00),
        currency: "EUR".into(),
    };

    let xml = EInvoiceGenerator::generate_ubl_xml(&doc);

    // Verify statutory Peppol BIS 3.0 specification identifiers
    assert!(xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"));
    assert!(
        xml.contains("urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0")
    );
    assert!(xml.contains("INV-2026-EU-9821"));
    assert!(xml.contains("DE123456789"));
    assert!(xml.contains("FR987654321"));
    assert!(xml.contains("17850.00"));
}

#[test]
fn test_golden_zatca_phase_2_hash_chaining_and_tlv_qr() {
    let ubl_xml_1 = b"<Invoice><ID>INV-KSA-001</ID><Total>1150.00</Total></Invoice>";
    let hash_1 = ZatcaPhase2Engine::compute_invoice_hash(ubl_xml_1, None);
    assert_eq!(hash_1.len(), 64); // SHA-256 hex string

    // Invoice 2 chains hash from Invoice 1
    let ubl_xml_2 = b"<Invoice><ID>INV-KSA-002</ID><Total>2300.00</Total></Invoice>";
    let hash_2 = ZatcaPhase2Engine::compute_invoice_hash(ubl_xml_2, Some(&hash_1));
    assert_eq!(hash_2.len(), 64);
    assert_ne!(hash_1, hash_2);

    // Generate ZATCA Phase 2 TLV QR Code (Base64)
    let qr_base64 = ZatcaPhase2Engine::generate_tlv_qr_base64(
        "Al-Mustaqbal Modern Tech LLC",
        "310123456700003",
        "2026-10-16T14:30:00Z",
        "2300.00",
        "300.00",
        &hash_2,
    );

    assert!(!qr_base64.is_empty());
    // Base64 chars check
    assert!(
        qr_base64
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    );
}

#[test]
fn test_golden_moadian_tax_id_and_canonical_payload() {
    let memory_id = "A12345"; // 6 alphanumeric memory ID
    let posting_dt = DateTime::<Utc>::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap(),
        Utc,
    );
    let serial = 10452;

    // 1. Generate 22-char unique Tax ID (شماره منحصر به فرد مالیاتی)
    let tax_id = MoadianTaxIdGenerator::generate_tax_id(memory_id, posting_dt, serial).unwrap();
    assert_eq!(tax_id.len(), 22);
    assert!(tax_id.starts_with("A12345"));

    // Verify Modulo 11 check digit verification passes
    assert!(MoadianTaxIdGenerator::verify_tax_id(&tax_id));

    // 2. Build Pattern 1 Sales Invoice Item Line (الگوی اول - فروش کالا و خدمات)
    // 10 units @ 10,000,000 Rials each = 100,000,000 Rials
    // Discount: 5,000,000 Rials -> Net: 95,000,000 Rials
    // VAT (10%): 9,500,000 Rials -> Total: 104,500,000 Rials
    let item = MoadianItemLine::new(
        "2720000123456", // Standard 13-digit goods/services code
        "Industrial Embedded Controller ARM-Cortex M4",
        dec!(10.0),
        dec!(10000000),
        dec!(5000000),
        dec!(10.0),
    );

    assert_eq!(item.prdis, dec!(100000000));
    assert_eq!(item.dis, dec!(5000000));
    assert_eq!(item.adis, dec!(95000000));
    assert_eq!(item.vam, dec!(9500000));
    assert_eq!(item.tsstam, dec!(104500000));

    // 3. Assemble complete Moadian Invoice Payload with automatic total aggregation
    let invoice = MoadianInvoicePayload::new(erp_trade::MoadianInvoiceParams {
        taxid: &tax_id,
        inno: "INV-2026-0010452",
        seller_tax_id: "10101234567",
        buyer_national_id: Some("14009876543"),
        buyer_tax_id: Some("41111222333"),
        posting_datetime: posting_dt,
        status: MoadianInvoiceStatus::Main,
        items: vec![item],
    })
    .expect("Valid Moadian invoice payload");

    assert_eq!(invoice.header.tprdis, dec!(100000000));
    assert_eq!(invoice.header.tdis, dec!(5000000));
    assert_eq!(invoice.header.tadis, dec!(95000000));
    assert_eq!(invoice.header.tvam, dec!(9500000));
    assert_eq!(invoice.header.tbill, dec!(104500000));

    // 4. Compute SHA-256 payload digest for INTA API digital signing
    let digest = invoice.compute_payload_hash();
    assert_eq!(digest.len(), 64);
}
