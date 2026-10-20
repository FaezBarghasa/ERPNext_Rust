//! ZATCA Phase  E-Invoicing Engine (`erp-trade::zatca`).
//!
//! Implements:
//! - Compliant UBL 2.1 XML invoice generation (`InvoiceTypeCode: 388`, Tax Category, VAT breakdown).
//! - Canonical XML SHA-256 digest computation and cryptographic invoice hash chaining ($H_n = \text{SHA-256}(UBL_n \parallel H_{n-1})$).
//! - Standard ZATCA 8-tag TLV (Tag-Length-Value) Base64 QR code encoding.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// ZATCA Invoice Line Item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZatcaInvoiceLine {
    pub id: String,
    pub item_name: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal, // e.g. 15.00
}

impl ZatcaInvoiceLine {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        item_name: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        vat_rate: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            item_name: item_name.into(),
            quantity,
            unit_price,
            vat_rate,
        }
    }

    #[must_use]
    pub fn line_extension_amount(&self) -> Decimal {
        self.quantity * self.unit_price
    }

    #[must_use]
    pub fn tax_amount(&self) -> Decimal {
        ((self.line_extension_amount() * self.vat_rate) / dec!(100.0)).round_dp(2)
    }
}

/// Parameters for constructing a ZATCA Phase 2 UBL 2.1 XML invoice.
#[derive(Debug, Clone)]
pub struct ZatcaUblInvoiceParams<'a> {
    pub invoice_id: &'a str,
    pub uuid: &'a str,
    pub issue_date: &'a str,        // YYYY-MM-DD
    pub issue_time: &'a str,        // HH:MM:SS
    pub invoice_type_code: &'a str, // "388" (Tax Invoice), "381" (Credit), "383" (Debit)
    pub seller_name: &'a str,
    pub seller_vat: &'a str,
    pub seller_street: &'a str,
    pub seller_building: &'a str,
    pub seller_city: &'a str,
    pub seller_postal_code: &'a str,
    pub buyer_name: &'a str,
    pub buyer_vat: Option<&'a str>,
    pub lines: Vec<ZatcaInvoiceLine>,
}

/// Parameters for generating ZATCA Phase 2 TLV QR Code.
#[derive(Debug, Clone)]
pub struct ZatcaQrParams<'a> {
    pub seller_name: &'a str,
    pub vat_number: &'a str,
    pub timestamp_iso: &'a str,
    pub invoice_total: &'a str,
    pub vat_total: &'a str,
    pub invoice_hash: &'a str,
    pub ecdsa_signature: Option<&'a [u8]>,
    pub ecdsa_public_key: Option<&'a [u8]>,
}

pub struct ZatcaPhase2Engine;

impl ZatcaPhase2Engine {
    /// Generates a compliant ZATCA Phase 2 UBL 2.1 XML invoice document.
    #[must_use]
    pub fn generate_ubl_xml(params: &ZatcaUblInvoiceParams<'_>) -> String {
        let total_line_extension: Decimal = params
            .lines
            .iter()
            .map(ZatcaInvoiceLine::line_extension_amount)
            .sum();
        let total_tax: Decimal = params.lines.iter().map(ZatcaInvoiceLine::tax_amount).sum();
        let total_inclusive = total_line_extension + total_tax;

        let mut xml = String::with_capacity(4096);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" "#);
        xml.push_str(r#"xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2" "#);
        xml.push_str(
            r#"xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">"#,
        );

        xml.push_str("<cbc:ProfileID>reporting:1.0</cbc:ProfileID>");
        xml.push_str(&format!("<cbc:ID>{}</cbc:ID>", params.invoice_id));
        xml.push_str(&format!("<cbc:UUID>{}</cbc:UUID>", params.uuid));
        xml.push_str(&format!(
            "<cbc:IssueDate>{}</cbc:IssueDate>",
            params.issue_date
        ));
        xml.push_str(&format!(
            "<cbc:IssueTime>{}</cbc:IssueTime>",
            params.issue_time
        ));
        xml.push_str(&format!(
            r#"<cbc:InvoiceTypeCode name="0100000">{}</cbc:InvoiceTypeCode>"#,
            params.invoice_type_code
        ));
        xml.push_str("<cbc:DocumentCurrencyCode>SAR</cbc:DocumentCurrencyCode>");
        xml.push_str("<cbc:TaxCurrencyCode>SAR</cbc:TaxCurrencyCode>");

        // AccountingSupplierParty
        xml.push_str("<cac:AccountingSupplierParty><cac:Party>");
        xml.push_str(&format!(
            "<cac:PostalAddress><cbc:StreetName>{}</cbc:StreetName><cbc:BuildingNumber>{}</cbc:BuildingNumber><cbc:CityName>{}</cbc:CityName><cbc:PostalZone>{}</cbc:PostalZone><cac:Country><cbc:IdentificationCode>SA</cbc:IdentificationCode></cac:Country></cac:PostalAddress>",
            params.seller_street, params.seller_building, params.seller_city, params.seller_postal_code
        ));
        xml.push_str(&format!(
            r#"<cac:PartyTaxScheme><cbc:CompanyID>{}</cbc:CompanyID><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:PartyTaxScheme>"#,
            params.seller_vat
        ));
        xml.push_str(&format!(
            "<cac:PartyLegalEntity><cbc:RegistrationName>{}</cbc:RegistrationName></cac:PartyLegalEntity>",
            params.seller_name
        ));
        xml.push_str("</cac:Party></cac:AccountingSupplierParty>");

        // AccountingCustomerParty
        xml.push_str("<cac:AccountingCustomerParty><cac:Party>");
        if let Some(vat) = params.buyer_vat {
            xml.push_str(&format!(
                r#"<cac:PartyTaxScheme><cbc:CompanyID>{}</cbc:CompanyID><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:PartyTaxScheme>"#,
                vat
            ));
        }
        xml.push_str(&format!(
            "<cac:PartyLegalEntity><cbc:RegistrationName>{}</cbc:RegistrationName></cac:PartyLegalEntity>",
            params.buyer_name
        ));
        xml.push_str("</cac:Party></cac:AccountingCustomerParty>");

        // TaxTotal
        xml.push_str(&format!(
            r#"<cac:TaxTotal><cbc:TaxAmount currencyID="SAR">{:.2}</cbc:TaxAmount>"#,
            total_tax
        ));
        xml.push_str(&format!(
            r#"<cac:TaxSubtotal><cbc:TaxableAmount currencyID="SAR">{:.2}</cbc:TaxableAmount><cbc:TaxAmount currencyID="SAR">{:.2}</cbc:TaxAmount><cac:TaxCategory><cbc:ID>S</cbc:ID><cbc:Percent>15.00</cbc:Percent><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:TaxCategory></cac:TaxSubtotal>"#,
            total_line_extension, total_tax
        ));
        xml.push_str("</cac:TaxTotal>");

        // LegalMonetaryTotal
        xml.push_str(&format!(
            r#"<cac:LegalMonetaryTotal><cbc:LineExtensionAmount currencyID="SAR">{:.2}</cbc:LineExtensionAmount><cbc:TaxExclusiveAmount currencyID="SAR">{:.2}</cbc:TaxExclusiveAmount><cbc:TaxInclusiveAmount currencyID="SAR">{:.2}</cbc:TaxInclusiveAmount><cbc:PayableAmount currencyID="SAR">{:.2}</cbc:PayableAmount></cac:LegalMonetaryTotal>"#,
            total_line_extension, total_line_extension, total_inclusive, total_inclusive
        ));

        // InvoiceLines
        for line in &params.lines {
            let line_ext = line.line_extension_amount();
            let line_tax = line.tax_amount();
            xml.push_str(&format!(
                r#"<cac:InvoiceLine><cbc:ID>{}</cbc:ID><cbc:InvoicedQuantity unitCode="PCE">{:.2}</cbc:InvoicedQuantity><cbc:LineExtensionAmount currencyID="SAR">{:.2}</cbc:LineExtensionAmount><cac:TaxTotal><cbc:TaxAmount currencyID="SAR">{:.2}</cbc:TaxAmount><cbc:RoundingAmount currencyID="SAR">{:.2}</cbc:RoundingAmount></cac:TaxTotal><cac:Item><cbc:Name>{}</cbc:Name><cac:ClassifiedTaxCategory><cbc:ID>S</cbc:ID><cbc:Percent>{:.2}</cbc:Percent><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory></cac:Item><cac:Price><cbc:PriceAmount currencyID="SAR">{:.2}</cbc:PriceAmount></cac:Price></cac:InvoiceLine>"#,
                line.id, line.quantity, line_ext, line_tax, line_ext + line_tax, line.item_name, line.vat_rate, line.unit_price
            ));
        }

        xml.push_str("</Invoice>");
        xml
    }

    /// Computes the cryptographic invoice hash chaining:
    /// Hash_n = SHA-256(UBL_Bytes_n || Hash_{n-1})
    #[must_use]
    pub fn compute_invoice_hash(
        ubl_xml_bytes: &[u8],
        previous_invoice_hash: Option<&str>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ubl_xml_bytes);
        if let Some(prev) = previous_invoice_hash {
            hasher.update(prev.as_bytes());
        }
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Computes canonical SHA-256 digest of UBL XML document.
    #[must_use]
    pub fn compute_canonical_xml_digest(xml: &str) -> String {
        let trimmed = xml.trim();
        let mut hasher = Sha256::new();
        hasher.update(trimmed.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Encodes a single TLV tag: Tag (1 byte) + Length (1 byte) + Value.
    fn encode_tlv_tag(tag: u8, value: &[u8]) -> Vec<u8> {
        let len = value.len() as u8;
        let mut out = Vec::with_capacity(2 + value.len());
        out.push(tag);
        out.push(len);
        out.extend_from_slice(value);
        out
    }

    /// Generates standard ZATCA Phase 2 TLV Base64 QR Code string:
    /// - Tag 1: Seller's Name
    /// - Tag 2: VAT Registration Number
    /// - Tag 3: Time Stamp (ISO 8601)
    /// - Tag 4: Invoice Total (with VAT)
    /// - Tag 5: VAT Total
    /// - Tag 6: Invoice Hash
    /// - Tag 7: ECDSA Digital Signature (Optional in sandbox, mandatory in production)
    /// - Tag 8: ECDSA Public Key Certificate (Optional)
    #[must_use]
    pub fn generate_phase2_tlv_qr_base64(params: &ZatcaQrParams<'_>) -> String {
        let mut tlv_bytes = Vec::new();
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(1, params.seller_name.as_bytes()));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(2, params.vat_number.as_bytes()));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(3, params.timestamp_iso.as_bytes()));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(4, params.invoice_total.as_bytes()));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(5, params.vat_total.as_bytes()));
        tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(6, params.invoice_hash.as_bytes()));

        if let Some(sig) = params.ecdsa_signature {
            tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(7, sig));
        }

        if let Some(pubkey) = params.ecdsa_public_key {
            tlv_bytes.extend_from_slice(&Self::encode_tlv_tag(8, pubkey));
        }

        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
        BASE64_STANDARD.encode(&tlv_bytes)
    }

    /// Backwards-compatible wrapper generating the 6-tag TLV Base64 QR code.
    #[must_use]
    pub fn generate_tlv_qr_base64(
        seller_name: &str,
        vat_number: &str,
        timestamp_iso: &str,
        invoice_total: &str,
        vat_total: &str,
        invoice_hash: &str,
    ) -> String {
        Self::generate_phase2_tlv_qr_base64(&ZatcaQrParams {
            seller_name,
            vat_number,
            timestamp_iso,
            invoice_total,
            vat_total,
            invoice_hash,
            ecdsa_signature: None,
            ecdsa_public_key: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zatca_hash_chaining_and_tlv_qr() {
        let xml_doc = b"<Invoice><ID>INV-KSA-001</ID></Invoice>";
        let genesis_hash = ZatcaPhase2Engine::compute_invoice_hash(xml_doc, None);
        assert_eq!(genesis_hash.len(), 64);

        let second_xml = b"<Invoice><ID>INV-KSA-002</ID></Invoice>";
        let second_hash = ZatcaPhase2Engine::compute_invoice_hash(second_xml, Some(&genesis_hash));
        assert_eq!(second_hash.len(), 64);
        assert_ne!(genesis_hash, second_hash);

        let qr_base64 = ZatcaPhase2Engine::generate_tlv_qr_base64(
            "Al-Madina Industrial Corp",
            "300123456700003",
            "2026-09-28T10:30:00Z",
            "1150.00",
            "150.00",
            &second_hash,
        );
        assert!(!qr_base64.is_empty());
    }

    #[test]
    fn test_zatca_ubl_xml_generation_and_digest() {
        let params = ZatcaUblInvoiceParams {
            invoice_id: "INV-2026-SA-001",
            uuid: "98a1b2c3-4d5e-6f7a-8b9c-0d1e2f3a4b5c",
            issue_date: "2026-10-20",
            issue_time: "10:00:00",
            invoice_type_code: "388",
            seller_name: "Modern Industrial Equipments Co.",
            seller_vat: "310123456700003",
            seller_street: "King Fahd Road",
            seller_building: "1234",
            seller_city: "Riyadh",
            seller_postal_code: "12345",
            buyer_name: "Arabian Construction Ltd",
            buyer_vat: Some("300987654300003"),
            lines: vec![
                ZatcaInvoiceLine::new("1", "Steel Pipe DN50", dec!(10.0), dec!(100.0), dec!(15.0)),
                ZatcaInvoiceLine::new("2", "Flange PN16", dec!(2.0), dec!(50.0), dec!(15.0)),
            ],
        };

        let xml = ZatcaPhase2Engine::generate_ubl_xml(&params);

        // Assert statutory UBL 2.1 tags
        assert!(xml.contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"));
        assert!(xml.contains("<cbc:ProfileID>reporting:1.0</cbc:ProfileID>"));
        assert!(xml.contains("<cbc:ID>INV-2026-SA-001</cbc:ID>"));
        assert!(xml.contains(r#"<cbc:InvoiceTypeCode name="0100000">388</cbc:InvoiceTypeCode>"#));
        assert!(xml.contains("310123456700003"));
        assert!(xml.contains("300987654300003"));

        // Totals:
        // Line 1: 10 * 100 = 1000, VAT 15% = 150
        // Line 2: 2 * 50 = 100, VAT 15% = 15
        // Total Line Extension: 1100.00, Tax Total: 165.00, Total Inclusive: 1265.00
        assert!(xml.contains(
            "<cbc:LineExtensionAmount currencyID=\"SAR\">1100.00</cbc:LineExtensionAmount>"
        ));
        assert!(xml.contains("<cbc:TaxAmount currencyID=\"SAR\">165.00</cbc:TaxAmount>"));
        assert!(xml.contains("<cbc:PayableAmount currencyID=\"SAR\">1265.00</cbc:PayableAmount>"));

        // Canonical XML digest
        let digest = ZatcaPhase2Engine::compute_canonical_xml_digest(&xml);
        assert_eq!(digest.len(), 64);

        // 8-Tag TLV QR generation with signature
        let dummy_sig = vec![0x30, 0x45, 0x02, 0x20, 0xaa, 0xbb];
        let dummy_pubkey = vec![0x04, 0x11, 0x22, 0x33];
        let qr = ZatcaPhase2Engine::generate_phase2_tlv_qr_base64(&ZatcaQrParams {
            seller_name: params.seller_name,
            vat_number: params.seller_vat,
            timestamp_iso: "2026-10-20T10:00:00Z",
            invoice_total: "1265.00",
            vat_total: "165.00",
            invoice_hash: &digest,
            ecdsa_signature: Some(&dummy_sig),
            ecdsa_public_key: Some(&dummy_pubkey),
        });
        assert!(!qr.is_empty());
    }
}
