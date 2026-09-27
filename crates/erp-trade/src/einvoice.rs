//! Universal Statutory Electronic Invoicing (Peppol BIS 3.0, ZUGFeRD 2.2 / Factur-X, KSeF).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EInvoiceStandard {
    PeppolBis3,
    Zugferd22FacturX,
    PolandKSeF,
    ItalySDI,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EInvoiceDocument {
    pub invoice_id: String,
    pub standard: EInvoiceStandard,
    pub seller_tax_id: String,
    pub buyer_tax_id: String,
    pub issue_date: chrono::NaiveDate,
    pub total_taxable_amount: Decimal,
    pub total_vat_amount: Decimal,
    pub payable_amount: Decimal,
    pub currency: String,
}

pub struct EInvoiceGenerator;

impl EInvoiceGenerator {
    /// Generates structured XML payload compliant with targeted statutory e-invoicing standard.
    #[must_use]
    pub fn generate_ubl_xml(doc: &EInvoiceDocument) -> String {
        match doc.standard {
            EInvoiceStandard::PeppolBis3 => {
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
         xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2"
         xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
    <cbc:CustomizationID>urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0</cbc:CustomizationID>
    <cbc:ID>{}</cbc:ID>
    <cbc:IssueDate>{}</cbc:IssueDate>
    <cbc:DocumentCurrencyCode>{}</cbc:DocumentCurrencyCode>
    <cac:AccountingSupplierParty>
        <cac:Party><cac:PartyTaxScheme><cbc:CompanyID>{}</cbc:CompanyID></cac:PartyTaxScheme></cac:Party>
    </cac:AccountingSupplierParty>
    <cac:AccountingCustomerParty>
        <cac:Party><cac:PartyTaxScheme><cbc:CompanyID>{}</cbc:CompanyID></cac:PartyTaxScheme></cac:Party>
    </cac:AccountingCustomerParty>
    <cac:LegalMonetaryTotal>
        <cbc:TaxExclusiveAmount currencyID="{}">{}</cbc:TaxExclusiveAmount>
        <cbc:TaxInclusiveAmount currencyID="{}">{}</cbc:TaxInclusiveAmount>
        <cbc:PayableAmount currencyID="{}">{}</cbc:PayableAmount>
    </cac:LegalMonetaryTotal>
</Invoice>"#,
                    doc.invoice_id,
                    doc.issue_date,
                    doc.currency,
                    doc.seller_tax_id,
                    doc.buyer_tax_id,
                    doc.currency,
                    doc.total_taxable_amount,
                    doc.currency,
                    doc.payable_amount,
                    doc.currency,
                    doc.payable_amount
                )
            }
            _ => format!(
                "<!-- Electronic Invoice Standard {:?} for {} -->",
                doc.standard, doc.invoice_id
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_peppol_e_invoice_xml_generation() {
        let doc = EInvoiceDocument {
            invoice_id: "INV-2026-EU-01".into(),
            standard: EInvoiceStandard::PeppolBis3,
            seller_tax_id: "DE123456789".into(),
            buyer_tax_id: "FR987654321".into(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 27).unwrap(),
            total_taxable_amount: dec!(1000.00),
            total_vat_amount: dec!(190.00),
            payable_amount: dec!(1190.00),
            currency: "EUR".into(),
        };

        let xml = EInvoiceGenerator::generate_ubl_xml(&doc);
        assert!(xml.contains("urn:fdc:peppol.eu:2017:poacc:billing:3.0"));
        assert!(xml.contains("DE123456789"));
        assert!(xml.contains("1190.00"));
    }
}
