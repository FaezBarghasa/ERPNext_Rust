/// Lightweight HTML invoice template renderer.
#[must_use]
pub fn render_invoice_html(invoice_no: &str, total_amount: &str, currency: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Invoice {invoice_no}</title>
<style>
body {{ font-family: sans-serif; margin: 40px; color: #333; }}
.header {{ text-align: center; border-bottom: 2px solid #ddd; padding-bottom: 20px; }}
.details {{ margin-top: 30px; font-size: 16px; }}
.total {{ margin-top: 30px; font-size: 20px; font-weight: bold; color: #111; }}
</style>
</head>
<body>
<div class="header">
  <h1>COMMERCIAL INVOICE</h1>
  <p>Invoice #: <strong>{invoice_no}</strong></p>
</div>
<div class="details">
  <p>Thank you for your business.</p>
</div>
<div class="total">
  <p>Grand Total: {currency} {total_amount}</p>
</div>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_invoice_html() {
        let html = render_invoice_html("ACC-INV-2026-00001", "1,250.00", "USD");
        assert!(html.contains("ACC-INV-2026-00001"));
        assert!(html.contains("USD 1,250.00"));
    }
}
