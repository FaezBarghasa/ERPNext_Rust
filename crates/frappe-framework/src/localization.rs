//! Global Multilingual, RTL & CJK Localization Matrix (`frappe_framework::localization`).
//!
//! Implements Pillar XXII: Persian/Arabic character & ZWNJ normalization,
//! Persian digits & currency formatters (IRR/IRT), Japan Qualified Invoice System
//! (*Tekikaku Seikyūsho*), Chinese Fapiao taxonomy, and CSS logical properties RTL engine.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Persian text normalizer for Arabic character drift and Zero-Width Non-Joiner (ZWNJ) preservation.
pub struct PersianNormalizer;

impl PersianNormalizer {
    /// Cleanses Arabic Yeh and Kaf to standard Persian forms (`ي` -> `ی`, `ك` -> `ک`)
    /// and normalizes duplicate spaces / ZWNJ characters (`\u{200C}`).
    #[must_use]
    pub fn normalize_persian_text(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                // Arabic Yeh variants -> Persian Yeh
                '\u{064A}' | '\u{0649}' | '\u{06CC}' => out.push('ی'),
                // Arabic Kaf -> Persian Kaf
                '\u{0643}' => out.push('ک'),
                // Arabic Heh with Yeh -> Heh + ZWNJ + Yeh
                '\u{06C0}' => {
                    out.push('ه');
                    out.push('\u{200C}');
                    out.push('ی');
                }
                // Preserve ZWNJ (نیم‌فاصله)
                '\u{200C}' => out.push('\u{200C}'),
                // Standard characters
                other => out.push(other),
            }
        }
        out
    }

    /// Converts ASCII Latin digits `0-9` into Persian digits `۰-۹`.
    #[must_use]
    pub fn to_persian_digits(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '0' => '۰',
                '1' => '۱',
                '2' => '۲',
                '3' => '۳',
                '4' => '۴',
                '5' => '۵',
                '6' => '۶',
                '7' => '۷',
                '8' => '۸',
                '9' => '۹',
                other => other,
            })
            .collect()
    }

    /// Formats an amount into Persian Rials / Tomans with Persian thousands separator (`٬`).
    #[must_use]
    pub fn format_toman_currency(amount: f64) -> CompactString {
        let formatted = format!("{:.0}", amount);
        let mut with_commas = String::new();
        let chars: Vec<char> = formatted.chars().rev().collect();

        for (i, &c) in chars.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                with_commas.push('٬');
            }
            with_commas.push(c);
        }

        let reversed: String = with_commas.chars().rev().collect();
        let persian_digits = Self::to_persian_digits(&reversed);
        format!("{persian_digits} تومان").into()
    }
}

/// Japanese Qualified Invoice System (*Tekikaku Seikyūsho* - インボイス制度) Validator.
pub struct JapanInvoiceEngine;

impl JapanInvoiceEngine {
    /// Validates Japanese Registered Business Number ($T + 13\text{ digits}$).
    #[must_use]
    pub fn validate_japan_tax_id(tax_id: &str) -> bool {
        let trimmed = tax_id.trim();
        if !trimmed.starts_with('T') && !trimmed.starts_with('t') {
            return false;
        }
        let digits = &trimmed[1..];
        digits.len() == 13 && digits.chars().all(|c| c.is_ascii_digit())
    }

    /// Computes dual 8% (reduced rate for food/beverages) and 10% (standard) consumption tax split.
    #[must_use]
    pub fn compute_japan_tax_split(
        standard_taxable: f64,
        reduced_taxable: f64,
    ) -> JapanTaxBreakdown {
        let standard_tax = (standard_taxable * 0.10).round();
        let reduced_tax = (reduced_taxable * 0.08).round();
        let total_tax = standard_tax + reduced_tax;
        let grand_total = standard_taxable + reduced_taxable + total_tax;

        JapanTaxBreakdown {
            standard_10_base: standard_taxable,
            standard_10_tax: standard_tax,
            reduced_8_base: reduced_taxable,
            reduced_8_tax: reduced_tax,
            total_tax,
            grand_total,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JapanTaxBreakdown {
    pub standard_10_base: f64,
    pub standard_10_tax: f64,
    pub reduced_8_base: f64,
    pub reduced_8_tax: f64,
    pub total_tax: f64,
    pub grand_total: f64,
}

/// Chinese Fapiao (发票) Taxonomy and Category Handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChineseFapiaoType {
    SpecialVatInvoice,  // 增值税专用发票
    GeneralVatInvoice,  // 增值税普通发票
    ElectronicGeneral,  // 增值税电子普通发票
    FullyDigitalized,   // 数电票 (All-in-one digital)
}

impl ChineseFapiaoType {
    /// Validates Chinese 18-character Unified Social Credit Code (USCC - 统一社会信用代码).
    #[must_use]
    pub fn validate_uscc(uscc: &str) -> bool {
        let trimmed = uscc.trim();
        trimmed.len() == 18
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() && c != 'I' && c != 'O' && c != 'Z' && c != 'S' && c != 'V')
    }
}

/// Dynamic CSS Logical Properties & RTL Direction Engine.
pub struct RtlEngine;

impl RtlEngine {
    /// Generates root CSS stylesheet enforcing BiDi logical flow.
    #[must_use]
    pub fn generate_rtl_css_prelude(is_rtl: bool) -> &'static str {
        if is_rtl {
            r#":root {
  direction: rtl;
  text-align: start;
  --margin-start: margin-right;
  --margin-end: margin-left;
  --padding-start: padding-right;
  --padding-end: padding-left;
  --font-body: 'Vazirmatn', 'Shabnam', sans-serif;
}"#
        } else {
            r#":root {
  direction: ltr;
  text-align: start;
  --margin-start: margin-left;
  --margin-end: margin-right;
  --padding-start: padding-left;
  --padding-end: padding-right;
  --font-body: 'Plus Jakarta Sans', sans-serif;
}"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persian_normalizer() {
        // Arabic yeh and kaf converted
        let raw = "شركت بازرگاني و خدماتي";
        let normalized = PersianNormalizer::normalize_persian_text(raw);
        assert_eq!(normalized, "شرکت بازرگانی و خدماتی");

        // Persian digits conversion
        assert_eq!(PersianNormalizer::to_persian_digits("1405/07/07"), "۱۴۰۵/۰۷/۰۷");

        // Toman currency formatter
        let toman_str = PersianNormalizer::format_toman_currency(2500000.0);
        assert_eq!(toman_str, "۲٬۵۰۰٬۰۰۰ تومان");
    }

    #[test]
    fn test_japan_tax_id_validation_and_split() {
        assert!(JapanInvoiceEngine::validate_japan_tax_id("T1234567890123"));
        assert!(!JapanInvoiceEngine::validate_japan_tax_id("1234567890123"));
        assert!(!JapanInvoiceEngine::validate_japan_tax_id("T123"));

        let split = JapanInvoiceEngine::compute_japan_tax_split(10000.0, 5000.0);
        assert_eq!(split.standard_10_tax, 1000.0);
        assert_eq!(split.reduced_8_tax, 400.0);
        assert_eq!(split.total_tax, 1400.0);
        assert_eq!(split.grand_total, 16400.0);
    }

    #[test]
    fn test_chinese_fapiao_uscc_validation() {
        // Standard 18-char USCC (Unified Social Credit Code)
        assert!(ChineseFapiaoType::validate_uscc("91110000000000001X"));
        assert!(!ChineseFapiaoType::validate_uscc("SHORT-CODE"));
    }
}
