//! Enterprise Commercial Print Format & Document Renderer (`erp-accounting::print_format`).
//!
//! Generates pixel-perfect, responsive HTML/CSS printable formats for:
//! - Sales Invoices & Credit Notes
//! - Purchase Orders & Purchase Invoices
//! - Delivery Notes & Goods Receipts
//!
//! Includes full VAT/GST tax tranches, bank wire details, signature seals, and `@media print` styling.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Company / Issuer information on printable letterhead.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableCompany {
    pub name: CompactString,
    pub tax_id: Option<CompactString>,
    pub address_line1: CompactString,
    pub address_line2: Option<CompactString>,
    pub city: CompactString,
    pub state_province: Option<CompactString>,
    pub postal_code: Option<CompactString>,
    pub country: CompactString,
    pub email: Option<CompactString>,
    pub phone: Option<CompactString>,
    pub website: Option<CompactString>,
}

/// Billed customer or supplier party details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableParty {
    pub name: CompactString,
    pub tax_id: Option<CompactString>,
    pub billing_address: CompactString,
    pub shipping_address: Option<CompactString>,
    pub contact_person: Option<CompactString>,
    pub email: Option<CompactString>,
    pub phone: Option<CompactString>,
}

/// Itemized invoice/order line item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableLineItem {
    pub item_code: CompactString,
    pub description: CompactString,
    pub uom: CompactString,
    pub quantity: Decimal,
    pub unit_rate: Decimal,
    pub discount_amount: Decimal,
    pub tax_rate_percent: Decimal,
    pub line_net_amount: Decimal,
}

/// Itemized tax charge or deduction breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableTaxCharge {
    pub title: CompactString,
    pub rate_percent: Decimal,
    pub tax_amount: Decimal,
}

/// Bank remittance and wire transfer details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableBankDetails {
    pub bank_name: CompactString,
    pub account_name: CompactString,
    pub account_number: CompactString,
    pub routing_or_swift: Option<CompactString>,
    pub iban: Option<CompactString>,
}

/// Complete commercial document model for print and PDF rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintableInvoice {
    pub document_title: CompactString,
    pub document_number: CompactString,
    pub posting_date: CompactString,
    pub due_date: Option<CompactString>,
    pub currency: CompactString,
    pub company: PrintableCompany,
    pub party: PrintableParty,
    pub line_items: Vec<PrintableLineItem>,
    pub subtotal: Decimal,
    pub total_discount: Decimal,
    pub tax_charges: Vec<PrintableTaxCharge>,
    pub grand_total: Decimal,
    pub outstanding_amount: Decimal,
    pub bank_details: Option<PrintableBankDetails>,
    pub payment_terms: Option<CompactString>,
    pub notes_and_terms: Option<CompactString>,
}

impl PrintableInvoice {
    /// Constructs a PrintableInvoice from a generic document JSON payload with sensible enterprise fallbacks.
    #[must_use]
    pub fn from_document_json(doctype: &str, doc: &serde_json::Value) -> Self {
        let doc_title = match doctype {
            "Sales Invoice" => "Commercial Sales Invoice",
            "Purchase Order" => "Official Purchase Order",
            "Purchase Invoice" => "Commercial Purchase Invoice",
            "Delivery Note" => "Goods Delivery Note",
            "Quotation" => "Sales Quotation",
            other => other,
        };

        let doc_number = doc.get("name").and_then(|v| v.as_str()).unwrap_or("DRAFT");

        let posting_date = doc
            .get("posting_date")
            .or_else(|| doc.get("transaction_date"))
            .and_then(|v| v.as_str())
            .unwrap_or("2026-10-12");

        let due_date = doc
            .get("due_date")
            .and_then(|v| v.as_str())
            .map(CompactString::from);

        let currency = doc
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("USD");

        let company_name = doc
            .get("company")
            .and_then(|v| v.as_str())
            .unwrap_or("Enterprise Organization");

        let party_name = doc
            .get("customer")
            .or_else(|| doc.get("supplier"))
            .or_else(|| doc.get("party_name"))
            .or_else(|| doc.get("customer_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Valued Client");

        let party_address = doc
            .get("address_display")
            .or_else(|| doc.get("billing_address"))
            .and_then(|v| v.as_str())
            .unwrap_or("Primary Address on File");

        let mut line_items = Vec::new();
        if let Some(items_arr) = doc.get("items").and_then(|v| v.as_array()) {
            for itm in items_arr {
                let code = itm
                    .get("item_code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ITEM-001");
                let desc = itm
                    .get("description")
                    .or_else(|| itm.get("item_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(code);
                let uom = itm
                    .get("uom")
                    .or_else(|| itm.get("stock_uom"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Nos");
                let qty = itm
                    .get("qty")
                    .or_else(|| itm.get("quantity"))
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ONE);
                let rate = itm
                    .get("rate")
                    .or_else(|| itm.get("unit_rate"))
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ZERO);
                let disc = itm
                    .get("discount_amount")
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ZERO);
                let tax_pct = itm
                    .get("tax_rate")
                    .or_else(|| itm.get("tax_rate_percent"))
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ZERO);
                let amount = itm
                    .get("amount")
                    .or_else(|| itm.get("net_amount"))
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| qty * rate - disc);

                line_items.push(PrintableLineItem {
                    item_code: code.into(),
                    description: desc.into(),
                    uom: uom.into(),
                    quantity: qty,
                    unit_rate: rate,
                    discount_amount: disc,
                    tax_rate_percent: tax_pct,
                    line_net_amount: amount,
                });
            }
        }

        let subtotal = doc
            .get("net_total")
            .or_else(|| doc.get("total"))
            .and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Decimal::from_f64_retain(n)
                } else if let Some(s) = v.as_str() {
                    s.parse::<Decimal>().ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| line_items.iter().map(|i| i.line_net_amount).sum());

        let total_discount = doc
            .get("discount_amount")
            .and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Decimal::from_f64_retain(n)
                } else if let Some(s) = v.as_str() {
                    s.parse::<Decimal>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(Decimal::ZERO);

        let mut tax_charges = Vec::new();
        if let Some(taxes_arr) = doc.get("taxes").and_then(|v| v.as_array()) {
            for t in taxes_arr {
                let title = t
                    .get("description")
                    .or_else(|| t.get("charge_type"))
                    .or_else(|| t.get("account_head"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Tax");
                let rate = t
                    .get("rate")
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ZERO);
                let amount = t
                    .get("tax_amount")
                    .and_then(|v| {
                        if let Some(n) = v.as_f64() {
                            Decimal::from_f64_retain(n)
                        } else if let Some(s) = v.as_str() {
                            s.parse::<Decimal>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Decimal::ZERO);
                tax_charges.push(PrintableTaxCharge {
                    title: title.into(),
                    rate_percent: rate,
                    tax_amount: amount,
                });
            }
        }

        let taxes_sum: Decimal = tax_charges.iter().map(|t| t.tax_amount).sum();
        let grand_total = doc
            .get("grand_total")
            .and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Decimal::from_f64_retain(n)
                } else if let Some(s) = v.as_str() {
                    s.parse::<Decimal>().ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| subtotal + taxes_sum - total_discount);

        let outstanding_amount = doc
            .get("outstanding_amount")
            .and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Decimal::from_f64_retain(n)
                } else if let Some(s) = v.as_str() {
                    s.parse::<Decimal>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(grand_total);

        Self {
            document_title: doc_title.into(),
            document_number: doc_number.into(),
            posting_date: posting_date.into(),
            due_date,
            currency: currency.into(),
            company: PrintableCompany {
                name: company_name.into(),
                tax_id: doc
                    .get("tax_id")
                    .and_then(|v| v.as_str())
                    .map(CompactString::from),
                address_line1: "Enterprise Way".into(),
                address_line2: None,
                city: "Corporate City".into(),
                state_province: None,
                postal_code: None,
                country: "United States".into(),
                email: None,
                phone: None,
                website: None,
            },
            party: PrintableParty {
                name: party_name.into(),
                tax_id: None,
                billing_address: party_address.into(),
                shipping_address: None,
                contact_person: None,
                email: None,
                phone: None,
            },
            line_items,
            subtotal,
            total_discount,
            tax_charges,
            grand_total,
            outstanding_amount,
            bank_details: None,
            payment_terms: doc
                .get("payment_terms_template")
                .and_then(|v| v.as_str())
                .map(CompactString::from),
            notes_and_terms: doc
                .get("terms")
                .and_then(|v| v.as_str())
                .map(CompactString::from),
        }
    }

    /// Renders commercial document into standalone responsive and print-ready HTML.
    #[must_use]
    pub fn render_html(&self) -> String {
        let mut items_html = String::with_capacity(self.line_items.len() * 256);
        for (idx, item) in self.line_items.iter().enumerate() {
            items_html.push_str(&format!(
                r#"<tr>
  <td class="text-center">{seq}</td>
  <td>
    <div class="fw-bold">{code}</div>
    <div class="text-muted small">{desc}</div>
  </td>
  <td class="text-center">{qty} {uom}</td>
  <td class="text-end">{rate:.2}</td>
  <td class="text-end text-muted">{disc:.2}</td>
  <td class="text-end">{tax_pct}%</td>
  <td class="text-end fw-bold">{net:.2}</td>
</tr>"#,
                seq = idx + 1,
                code = item.item_code,
                desc = item.description,
                qty = item.quantity,
                uom = item.uom,
                rate = item.unit_rate,
                disc = item.discount_amount,
                tax_pct = item.tax_rate_percent,
                net = item.line_net_amount
            ));
        }

        let mut taxes_html = String::new();
        for tax in &self.tax_charges {
            taxes_html.push_str(&format!(
                r#"<div class="d-flex justify-content-between py-1">
  <span class="text-muted">{title} ({rate}%):</span>
  <span class="fw-semibold">{curr} {amt:.2}</span>
</div>"#,
                title = tax.title,
                rate = tax.rate_percent,
                curr = self.currency,
                amt = tax.tax_amount
            ));
        }

        let bank_section = match &self.bank_details {
            Some(b) => format!(
                r#"<div class="bank-details p-3 rounded mt-3">
  <h6 class="fw-bold text-uppercase mb-2 text-primary">Remittance Bank Wire Instructions</h6>
  <div class="small"><strong>Bank:</strong> {bank}</div>
  <div class="small"><strong>Beneficiary:</strong> {name}</div>
  <div class="small"><strong>Account:</strong> {acc}</div>
  {swift}
  {iban}
</div>"#,
                bank = b.bank_name,
                name = b.account_name,
                acc = b.account_number,
                swift = b
                    .routing_or_swift
                    .as_ref()
                    .map_or(String::new(), |s| format!(
                        "<div class=\"small\"><strong>SWIFT / BIC:</strong> {s}</div>"
                    )),
                iban = b.iban.as_ref().map_or(String::new(), |i| format!(
                    "<div class=\"small\"><strong>IBAN:</strong> {i}</div>"
                ))
            ),
            None => String::new(),
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{doc_title} - {doc_num}</title>
<style>
  :root {{
    --primary: #1e3a8a;
    --border: #e2e8f0;
    --text-main: #0f172a;
    --text-muted: #64748b;
    --bg-light: #f8fafc;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: var(--text-main);
    background-color: #f1f5f9;
    margin: 0;
    padding: 24px;
    font-size: 14px;
    line-height: 1.5;
  }}
  .invoice-card {{
    background: #ffffff;
    max-width: 900px;
    margin: 0 auto;
    padding: 40px;
    border-radius: 8px;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }}
  .brand-header {{
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 2px solid var(--primary);
    padding-bottom: 24px;
    margin-bottom: 24px;
  }}
  .company-title {{
    font-size: 24px;
    font-weight: 800;
    color: var(--primary);
    letter-spacing: -0.5px;
    margin-bottom: 4px;
  }}
  .doc-meta-title {{
    font-size: 28px;
    font-weight: 800;
    color: var(--text-main);
    text-align: right;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }}
  .doc-num {{
    font-size: 16px;
    font-weight: 600;
    color: var(--primary);
    text-align: right;
  }}
  .party-grid {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
    margin-bottom: 32px;
  }}
  .section-label {{
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.5px;
    margin-bottom: 6px;
  }}
  table.items-table {{
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 24px;
  }}
  table.items-table th {{
    background-color: var(--bg-light);
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    padding: 12px 10px;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-muted);
  }}
  table.items-table td {{
    padding: 12px 10px;
    border-bottom: 1px solid var(--border);
  }}
  .text-center {{ text-align: center; }}
  .text-end {{ text-align: right; }}
  .fw-bold {{ font-weight: 700; }}
  .fw-semibold {{ font-weight: 600; }}
  .text-muted {{ color: var(--text-muted); }}
  .small {{ font-size: 12px; }}
  .totals-container {{
    display: flex;
    justify-content: flex-end;
    margin-bottom: 32px;
  }}
  .totals-box {{
    width: 340px;
    background: var(--bg-light);
    padding: 16px 20px;
    border-radius: 6px;
    border: 1px solid var(--border);
  }}
  .grand-total-row {{
    display: flex;
    justify-content: space-between;
    border-top: 2px solid var(--primary);
    padding-top: 8px;
    margin-top: 8px;
    font-size: 16px;
    font-weight: 800;
    color: var(--primary);
  }}
  .bank-details {{
    background: #eff6ff;
    border: 1px solid #bfdbfe;
    padding: 16px;
    border-radius: 6px;
  }}
  .signature-grid {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 40px;
    margin-top: 48px;
    padding-top: 24px;
  }}
  .signature-line {{
    border-top: 1px solid var(--text-muted);
    padding-top: 8px;
    text-align: center;
    font-size: 12px;
    color: var(--text-muted);
  }}
  @media print {{
    body {{ background: #ffffff; padding: 0; }}
    .invoice-card {{ box-shadow: none; padding: 0; max-width: 100%; }}
    .no-print {{ display: none !important; }}
    @page {{ margin: 1.5cm; }}
  }}
</style>
</head>
<body>
<div class="invoice-card">
  <div class="no-print" style="margin-bottom: 20px; display: flex; justify-content: flex-end; gap: 12px;">
    <button onclick="window.print()" style="padding: 8px 16px; background: var(--primary); color: white; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 8px; transition: transform 0.15s ease;">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9V2h12v7M6 18H4a2 2 0 01-2-2v-5a2 2 0 012-2h16a2 2 0 012 2v5a2 2 0 01-2 2h-2"/><path d="M6 14h12v8H6z"/></svg>
      Print Commercial Document
    </button>
  </div>

  <div class="brand-header">
    <div>
      <div class="company-title">{comp_name}</div>
      <div class="small text-muted">{comp_addr1}</div>
      <div class="small text-muted">{comp_city}, {comp_country}</div>
      {comp_tax}
      {comp_contact}
    </div>
    <div>
      <div class="doc-meta-title">{doc_title}</div>
      <div class="doc-num">{doc_num}</div>
      <div class="small text-muted" style="text-align: right; margin-top: 6px;">
        <strong>Date:</strong> {date}
      </div>
      {due_date_str}
    </div>
  </div>

  <div class="party-grid">
    <div>
      <div class="section-label">Billed To / Debtor</div>
      <div class="fw-bold" style="font-size: 15px;">{party_name}</div>
      <div class="small text-muted">{party_bill}</div>
      {party_tax_str}
      {party_contact_str}
    </div>
    <div>
      <div class="section-label">Shipping / Delivery Destination</div>
      <div class="small">{ship_address}</div>
      {terms_str}
    </div>
  </div>

  <table class="items-table">
    <thead>
      <tr>
        <th class="text-center" style="width: 40px;">#</th>
        <th>Item Description</th>
        <th class="text-center" style="width: 100px;">Quantity</th>
        <th class="text-end" style="width: 110px;">Unit Rate</th>
        <th class="text-end" style="width: 90px;">Discount</th>
        <th class="text-end" style="width: 80px;">Tax %</th>
        <th class="text-end" style="width: 120px;">Amount ({currency})</th>
      </tr>
    </thead>
    <tbody>
      {items_html}
    </tbody>
  </table>

  <div class="totals-container">
    <div class="totals-box">
      <div class="d-flex justify-content-between py-1" style="display: flex; justify-content: space-between;">
        <span class="text-muted">Item Subtotal:</span>
        <span class="fw-semibold">{currency} {subtotal:.2}</span>
      </div>
      <div class="d-flex justify-content-between py-1" style="display: flex; justify-content: space-between;">
        <span class="text-muted">Total Discount:</span>
        <span class="text-muted">-{currency} {discount:.2}</span>
      </div>
      {taxes_html}
      <div class="grand-total-row">
        <span>Grand Total:</span>
        <span>{currency} {grand_total:.2}</span>
      </div>
      <div style="display: flex; justify-content: space-between; margin-top: 4px; font-size: 13px;">
        <span class="text-muted">Outstanding Balance:</span>
        <span class="fw-bold" style="color: #b91c1c;">{currency} {outstanding:.2}</span>
      </div>
    </div>
  </div>

  {bank_section}

  {notes_section}

  <div class="signature-grid">
    <div>
      <div class="signature-line">Authorized Signature & Company Seal</div>
    </div>
    <div>
      <div class="signature-line">Customer Acceptance & Acknowledgment</div>
    </div>
  </div>
</div>
</body>
</html>"#,
            doc_title = self.document_title,
            doc_num = self.document_number,
            comp_name = self.company.name,
            comp_addr1 = self.company.address_line1,
            comp_city = self.company.city,
            comp_country = self.company.country,
            comp_tax = self
                .company
                .tax_id
                .as_ref()
                .map_or(String::new(), |t| format!("<div class=\"small text-muted\"><strong>Tax / VAT ID:</strong> {t}</div>")),
            comp_contact = self
                .company
                .email
                .as_ref()
                .map_or(String::new(), |e| format!("<div class=\"small text-muted\">{e}</div>")),
            date = self.posting_date,
            due_date_str = self
                .due_date
                .as_ref()
                .map_or(String::new(), |d| format!("<div class=\"small text-muted\" style=\"text-align: right;\"><strong>Due Date:</strong> {d}</div>")),
            party_name = self.party.name,
            party_bill = self.party.billing_address,
            party_tax_str = self
                .party
                .tax_id
                .as_ref()
                .map_or(String::new(), |t| format!("<div class=\"small text-muted\"><strong>Tax ID:</strong> {t}</div>")),
            party_contact_str = self
                .party
                .email
                .as_ref()
                .map_or(String::new(), |e| format!("<div class=\"small text-muted\">{e}</div>")),
            ship_address = self
                .party
                .shipping_address
                .as_ref()
                .unwrap_or(&self.party.billing_address),
            terms_str = self
                .payment_terms
                .as_ref()
                .map_or(String::new(), |t| format!("<div class=\"small text-muted mt-2\"><strong>Payment Terms:</strong> {t}</div>")),
            items_html = items_html,
            currency = self.currency,
            subtotal = self.subtotal,
            discount = self.total_discount,
            taxes_html = taxes_html,
            grand_total = self.grand_total,
            outstanding = self.outstanding_amount,
            bank_section = bank_section,
            notes_section = self.notes_and_terms.as_ref().map_or(String::new(), |n| {
                format!(
                    r#"<div style="margin-top: 24px; font-size: 12px; color: var(--text-muted);">
  <strong>Terms & Conditions:</strong> {n}
</div>"#
                )
            })
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_printable_invoice_html_rendering() {
        let invoice = PrintableInvoice {
            document_title: "Commercial Sales Invoice".into(),
            document_number: "ACC-SINV-2026-00042".into(),
            posting_date: "2026-10-12".into(),
            due_date: Some("2026-11-12".into()),
            currency: "USD".into(),
            company: PrintableCompany {
                name: "Apex Precision Dynamics Corp".into(),
                tax_id: Some("US-EIN-99-8877665".into()),
                address_line1: "100 Industrial Parkway".into(),
                address_line2: Some("Suite 400".into()),
                city: "Austin".into(),
                state_province: Some("TX".into()),
                postal_code: Some("78701".into()),
                country: "United States".into(),
                email: Some("billing@apex-dynamics.com".into()),
                phone: Some("+1-512-555-0199".into()),
                website: Some("https://apex-dynamics.com".into()),
            },
            party: PrintableParty {
                name: "Horizon Aero Manufacturing".into(),
                tax_id: Some("US-EIN-11-2233445".into()),
                billing_address: "450 Boeing Field Road, Seattle, WA 98108".into(),
                shipping_address: Some("Dock 4, 450 Boeing Field Road, Seattle, WA 98108".into()),
                contact_person: Some("Marcus Vance".into()),
                email: Some("purchasing@horizon-aero.com".into()),
                phone: Some("+1-206-555-0182".into()),
            },
            line_items: vec![
                PrintableLineItem {
                    item_code: "TURB-TI-880".into(),
                    description: "Titanium Alloy Compressor Disc".into(),
                    uom: "Pcs".into(),
                    quantity: dec!(4),
                    unit_rate: dec!(12500.00),
                    discount_amount: dec!(1000.00),
                    tax_rate_percent: dec!(8.25),
                    line_net_amount: dec!(49000.00),
                },
                PrintableLineItem {
                    item_code: "SEAL-RING-N9".into(),
                    description: "High-Temperature Ceramic Seal Ring".into(),
                    uom: "Sets".into(),
                    quantity: dec!(8),
                    unit_rate: dec!(450.00),
                    discount_amount: dec!(0.00),
                    tax_rate_percent: dec!(8.25),
                    line_net_amount: dec!(3600.00),
                },
            ],
            subtotal: dec!(53600.00),
            total_discount: dec!(1000.00),
            tax_charges: vec![PrintableTaxCharge {
                title: "State & County Sales Tax".into(),
                rate_percent: dec!(8.25),
                tax_amount: dec!(4339.50),
            }],
            grand_total: dec!(56939.50),
            outstanding_amount: dec!(56939.50),
            bank_details: Some(PrintableBankDetails {
                bank_name: "JPMorgan Chase Bank, N.A.".into(),
                account_name: "Apex Precision Dynamics Corp".into(),
                account_number: "9876543210".into(),
                routing_or_swift: Some("CHASUS33".into()),
                iban: Some("US64CHAS9876543210".into()),
            }),
            payment_terms: Some("Net 30 Days from Receipt of Goods".into()),
            notes_and_terms: Some(
                "Late payments subject to 1.5% statutory monthly interest.".into(),
            ),
        };

        let html = invoice.render_html();
        assert!(html.contains("ACC-SINV-2026-00042"));
        assert!(html.contains("Apex Precision Dynamics Corp"));
        assert!(html.contains("Horizon Aero Manufacturing"));
        assert!(html.contains("TURB-TI-880"));
        assert!(html.contains("56939.50"));
        assert!(html.contains("CHASUS33"));
        assert!(html.contains("@media print"));
    }

    #[test]
    fn test_printable_invoice_from_document_json() {
        let doc_json = serde_json::json!({
            "name": "ACC-SINV-2026-00099",
            "doctype": "Sales Invoice",
            "posting_date": "2026-10-12",
            "due_date": "2026-11-12",
            "company": "RustNext Global Enterprise",
            "customer": "Starlight Aerospace",
            "currency": "EUR",
            "items": [
                {
                    "item_code": "AV-SENSOR-01",
                    "description": "Avionics Multi-Spectrum Sensor",
                    "qty": 2.0,
                    "rate": 4500.0,
                    "discount_amount": 0.0,
                    "tax_rate": 19.0,
                    "amount": 9000.0
                }
            ],
            "taxes": [
                {
                    "description": "EU Standard VAT",
                    "rate": 19.0,
                    "tax_amount": 1710.0
                }
            ],
            "net_total": 9000.0,
            "grand_total": 10710.0,
            "outstanding_amount": 10710.0
        });

        let printable = PrintableInvoice::from_document_json("Sales Invoice", &doc_json);
        assert_eq!(printable.document_number, "ACC-SINV-2026-00099");
        assert_eq!(printable.party.name, "Starlight Aerospace");
        assert_eq!(printable.currency, "EUR");
        assert_eq!(printable.line_items.len(), 1);
        assert_eq!(printable.grand_total, dec!(10710.0));

        let html = printable.render_html();
        assert!(html.contains("ACC-SINV-2026-00099"));
        assert!(html.contains("Starlight Aerospace"));
        assert!(html.contains("Avionics Multi-Spectrum Sensor"));
        assert!(html.contains("10710.00"));
    }
}
