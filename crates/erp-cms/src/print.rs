//! Pure-Rust Typst PDF/A rendering (Stage 6.2): replaces headless Chrome.
//! Stub using typst's compile + pdf pipeline (actual API in 0.15+).
use typst::typst::{Library, World};

/// Minimal invoice template -> PDF bytes.
pub fn render_invoice_pdf(invoice_no: &str, total_cents: i64) -> Vec<u8> {
    let src = format!("#set page(width: 210mm, height: 297mm, margin: 20mm)\n#set text(font: \"DejaVu Sans\")\n#align(center)[= INVOICE {}]\n#grid(columns: 2, gutter: 10pt)[\n  Invoice #: {}\n  Total: {} \n]", invoice_no, invoice_no, total_cents as f64 / 100.0);
    // Real implementation: typst::compile + typst::pdf
    // For now return minimal PDF to validate integration compiles
    format!("%PDF-1.4\n%Typed\n%%EOF\n").into_bytes()
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn pdf_nonempty() {
        let pdf = render_invoice_pdf("INV-001", 12345);
        assert!(!pdf.is_empty());
        assert!(pdf.starts_with(b"%PDF"));
    }
}
