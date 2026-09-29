use crate::jalali::JalaliDate;
use chrono::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Document Line Item for Print Rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintLineItem {
    pub item_code: String,
    pub description: String,
    pub qty: Decimal,
    pub unit_price: Decimal,
    pub discount_pct: Decimal,
    pub tax_rate_pct: Decimal,
    pub line_total: Decimal,
}

/// Print Formatting Context for Commercial Documents (Invoices, Orders, Quotes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoicePrintContext {
    pub document_title: String,
    pub document_number: String,
    pub posting_date: String,
    pub due_date: Option<String>,
    pub company_name: String,
    pub company_tax_id: String,
    pub company_address: String,
    pub customer_name: String,
    pub customer_tax_id: Option<String>,
    pub customer_address: String,
    pub currency: String,
    pub items: Vec<PrintLineItem>,
    pub net_total: Decimal,
    pub total_discount: Decimal,
    pub total_tax: Decimal,
    pub grand_total: Decimal,
    pub qr_data: Option<String>,
    pub terms_and_conditions: Option<String>,
}

/// Thermal 80mm/58mm POS Receipt Context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptPrintContext {
    pub store_name: String,
    pub terminal_id: String,
    pub cashier_name: String,
    pub receipt_number: String,
    pub timestamp: String,
    pub items: Vec<PrintLineItem>,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub payment_method: String,
    pub change_due: Decimal,
}

/// Print Format Renderer.
pub struct PrintEngine;

impl PrintEngine {
    /// Renders a responsive, print-optimized HTML invoice with styling and CSS paged media.
    #[must_use]
    pub fn render_html_invoice(ctx: &InvoicePrintContext) -> String {
        let jalali_note = if let Ok(dt) = DateTime::parse_from_rfc3339(&ctx.posting_date) {
            let j = JalaliDate::from_gregorian(dt.naive_utc().date());
            format!(" / Jalali: {}", j.format_shamsi())
        } else {
            String::new()
        };

        let mut items_html = String::new();
        for (idx, item) in ctx.items.iter().enumerate() {
            items_html.push_str(&format!(
                r#"<tr>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb; text-align: center;">{}</td>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;"><strong>{}</strong><br><small style="color: #6b7280;">{}</small></td>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb; text-align: right;">{}</td>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb; text-align: right;">{} {}</td>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb; text-align: right;">{}%</td>
                    <td style="padding: 8px; border-bottom: 1px solid #e5e7eb; text-align: right;"><strong>{} {}</strong></td>
                </tr>"#,
                idx + 1,
                item.item_code,
                item.description,
                item.qty,
                item.unit_price,
                ctx.currency,
                item.discount_pct,
                item.line_total,
                ctx.currency
            ));
        }

        let qr_section = if let Some(ref qr) = ctx.qr_data {
            format!(
                r#"<div style="text-align: center; margin-top: 20px;">
                    <div style="display: inline-block; padding: 10px; border: 1px solid #d1d5db; border-radius: 6px; background: #fff;">
                        <span style="font-size: 11px; font-family: monospace; color: #374151;">QR: {}</span>
                    </div>
                </div>"#,
                qr
            )
        } else {
            String::new()
        };

        let terms = ctx.terms_and_conditions.as_deref().unwrap_or("Payment due within 30 days. Thank you for your business.");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} - {}</title>
    <style>
        @page {{ size: A4; margin: 20mm; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; color: #111827; background: #fff; margin: 0; padding: 24px; font-size: 13px; line-height: 1.5; }}
        .header {{ display: flex; justify-content: space-between; align-items: flex-start; border-bottom: 2px solid #3b82f6; padding-bottom: 16px; margin-bottom: 24px; }}
        .company-title {{ font-size: 20px; font-weight: 700; color: #1e40af; margin: 0 0 4px 0; }}
        .doc-title {{ font-size: 24px; font-weight: 800; text-transform: uppercase; color: #1f2937; margin: 0 0 4px 0; }}
        .meta-grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 24px; margin-bottom: 24px; }}
        .meta-card {{ background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; padding: 14px; }}
        .table-wrap {{ width: 100%; border-collapse: collapse; margin-bottom: 24px; }}
        .table-wrap th {{ background: #f3f4f6; color: #374151; font-weight: 600; text-align: left; padding: 10px 8px; border-bottom: 2px solid #d1d5db; }}
        .summary-card {{ width: 280px; margin-left: auto; background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; padding: 16px; }}
        .summary-row {{ display: flex; justify-content: space-between; margin-bottom: 6px; font-size: 13px; }}
        .summary-grand {{ border-top: 2px solid #d1d5db; padding-top: 8px; font-size: 15px; font-weight: 700; color: #1e40af; }}
        .footer {{ margin-top: 36px; padding-top: 16px; border-top: 1px solid #e5e7eb; font-size: 11px; color: #6b7280; }}
    </style>
</head>
<body>
    <div class="header">
        <div>
            <h1 class="company-title">{}</h1>
            <div>Tax ID / VAT: <strong>{}</strong></div>
            <div>{}</div>
        </div>
        <div style="text-align: right;">
            <h2 class="doc-title">{}</h2>
            <div>Doc #: <strong>{}</strong></div>
            <div>Date: {}{}</div>
        </div>
    </div>

    <div class="meta-grid">
        <div class="meta-card">
            <h4 style="margin: 0 0 6px 0; color: #4b5563; text-transform: uppercase; font-size: 11px;">Billed To</h4>
            <div style="font-size: 14px; font-weight: 600;">{}</div>
            <div>Tax ID: {}</div>
            <div>{}</div>
        </div>
        <div class="meta-card">
            <h4 style="margin: 0 0 6px 0; color: #4b5563; text-transform: uppercase; font-size: 11px;">Payment & Delivery</h4>
            <div>Currency: <strong>{}</strong></div>
            <div>Due Date: {}</div>
        </div>
    </div>

    <table class="table-wrap">
        <thead>
            <tr>
                <th style="width: 40px; text-align: center;">#</th>
                <th>Item & Description</th>
                <th style="width: 70px; text-align: right;">Qty</th>
                <th style="width: 100px; text-align: right;">Rate</th>
                <th style="width: 70px; text-align: right;">Disc.</th>
                <th style="width: 120px; text-align: right;">Amount</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <div class="summary-card">
        <div class="summary-row">
            <span>Net Subtotal:</span>
            <span>{} {}</span>
        </div>
        <div class="summary-row">
            <span>Total Discount:</span>
            <span>-{} {}</span>
        </div>
        <div class="summary-row">
            <span>Tax / VAT:</span>
            <span>{} {}</span>
        </div>
        <div class="summary-row summary-grand">
            <span>Grand Total:</span>
            <span>{} {}</span>
        </div>
    </div>

    {}

    <div class="footer">
        <strong>Terms & Conditions:</strong><br>
        {}
    </div>
</body>
</html>"#,
            ctx.document_title,
            ctx.document_number,
            ctx.company_name,
            ctx.company_tax_id,
            ctx.company_address,
            ctx.document_title,
            ctx.document_number,
            ctx.posting_date,
            jalali_note,
            ctx.customer_name,
            ctx.customer_tax_id.as_deref().unwrap_or("N/A"),
            ctx.customer_address,
            ctx.currency,
            ctx.due_date.as_deref().unwrap_or("On Receipt"),
            items_html,
            ctx.net_total,
            ctx.currency,
            ctx.total_discount,
            ctx.currency,
            ctx.total_tax,
            ctx.currency,
            ctx.grand_total,
            ctx.currency,
            qr_section,
            terms
        )
    }

    /// Renders an ultra-compact, thermal receipt format (ESC/POS style plain text/HTML).
    #[must_use]
    pub fn render_thermal_receipt(ctx: &ReceiptPrintContext) -> String {
        let mut lines = String::new();
        lines.push_str(&format!("================================\n"));
        lines.push_str(&format!("{:^32}\n", ctx.store_name));
        lines.push_str(&format!("{:^32}\n", format!("Term: {} | Cashier: {}", ctx.terminal_id, ctx.cashier_name)));
        lines.push_str(&format!("{:^32}\n", ctx.receipt_number));
        lines.push_str(&format!("{:^32}\n", ctx.timestamp));
        lines.push_str(&format!("--------------------------------\n"));
        lines.push_str(&format!("{:<16} {:>4} {:>10}\n", "Item", "Qty", "Price"));
        lines.push_str(&format!("--------------------------------\n"));

        for item in &ctx.items {
            let item_name = if item.item_code.len() > 16 {
                &item.item_code[..16]
            } else {
                &item.item_code
            };
            lines.push_str(&format!("{:<16} {:>4} {:>10.2}\n", item_name, item.qty, item.line_total));
        }

        lines.push_str(&format!("--------------------------------\n"));
        lines.push_str(&format!("{:<20} {:>11.2}\n", "Subtotal:", ctx.subtotal));
        lines.push_str(&format!("{:<20} {:>11.2}\n", "Tax:", ctx.tax));
        lines.push_str(&format!("{:<20} {:>11.2}\n", "TOTAL:", ctx.total));
        lines.push_str(&format!("--------------------------------\n"));
        lines.push_str(&format!("{:<20} {:>11}\n", "Paid with:", ctx.payment_method));
        lines.push_str(&format!("{:<20} {:>11.2}\n", "Change Due:", ctx.change_due));
        lines.push_str(&format!("================================\n"));
        lines.push_str(&format!("{:^32}\n", "Thank you for visiting!"));
        lines.push_str(&format!("================================\n"));

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_html_invoice_rendering() {
        let ctx = InvoicePrintContext {
            document_title: "Tax Invoice".to_string(),
            document_number: "INV-2026-0001".to_string(),
            posting_date: "2026-09-30T10:00:00Z".to_string(),
            due_date: Some("2026-10-30".to_string()),
            company_name: "Apex Technologies Corp".to_string(),
            company_tax_id: "TAX-99887711".to_string(),
            company_address: "Tech Hub 1, Innovation Way".to_string(),
            customer_name: "Faez Barghasa".to_string(),
            customer_tax_id: Some("CUST-TAX-123".to_string()),
            customer_address: "42 Systems Ave".to_string(),
            currency: "USD".to_string(),
            items: vec![PrintLineItem {
                item_code: "HARDWARE-SERVER-1U".to_string(),
                description: "Rust Edge Server 1U Rack".to_string(),
                qty: dec!(2),
                unit_price: dec!(1500.00),
                discount_pct: dec!(10),
                tax_rate_pct: dec!(15),
                line_total: dec!(2700.00),
            }],
            net_total: dec!(3000.00),
            total_discount: dec!(300.00),
            total_tax: dec!(405.00),
            grand_total: dec!(3105.00),
            qr_data: Some("ZATCA_BASE64_QR_CODE_PROD_123".to_string()),
            terms_and_conditions: Some("Strict 30 days payment term.".to_string()),
        };

        let html = PrintEngine::render_html_invoice(&ctx);
        assert!(html.contains("Tax Invoice"));
        assert!(html.contains("INV-2026-0001"));
        assert!(html.contains("Apex Technologies Corp"));
        assert!(html.contains("HARDWARE-SERVER-1U"));
        assert!(html.contains("3105.00"));
        assert!(html.contains("ZATCA_BASE64_QR_CODE_PROD_123"));
    }

    #[test]
    fn test_thermal_receipt_rendering() {
        let ctx = ReceiptPrintContext {
            store_name: "RustNext POS #1".to_string(),
            terminal_id: "POS-01".to_string(),
            cashier_name: "Alice".to_string(),
            receipt_number: "RCP-887766".to_string(),
            timestamp: "2026-09-30 14:30:00".to_string(),
            items: vec![PrintLineItem {
                item_code: "Coffee Beans 1kg".to_string(),
                description: "Arabica Roast".to_string(),
                qty: dec!(1),
                unit_price: dec!(25.00),
                discount_pct: dec!(0),
                tax_rate_pct: dec!(0),
                line_total: dec!(25.00),
            }],
            subtotal: dec!(25.00),
            discount: dec!(0.00),
            tax: dec!(2.50),
            total: dec!(27.50),
            payment_method: "NFC / Card".to_string(),
            change_due: dec!(0.00),
        };

        let text = PrintEngine::render_thermal_receipt(&ctx);
        assert!(text.contains("RustNext POS #1"));
        assert!(text.contains("Coffee Beans 1kg"));
        assert!(text.contains("27.50"));
    }
}
