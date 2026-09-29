//! Format Decoding & Encoding Detection (`erp_importer::format_decoder`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Supported tabular file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestionFormat {
    Csv,
    Tsv,
    Xlsx,
    Xls,
    Ods,
    JsonArray,
}

/// Detected text encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    ShiftJis,
    Gbk,
    Cp1256PersianArabic,
}

/// Parsed raw tabular row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawDataRow {
    pub row_index: usize,
    pub fields: Vec<CompactString>,
}

/// Decoder for streaming raw tabular data.
pub struct FormatDecoder;

impl FormatDecoder {
    /// Detects character encoding from initial byte magic or BOM.
    #[must_use]
    pub fn detect_encoding(bytes: &[u8]) -> CharacterEncoding {
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            return CharacterEncoding::Utf8;
        }
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            return CharacterEncoding::Utf16Le;
        }
        if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
            return CharacterEncoding::Utf16Be;
        }

        // Check if valid UTF-8
        if std::str::from_utf8(bytes).is_ok() {
            CharacterEncoding::Utf8
        } else {
            CharacterEncoding::Cp1256PersianArabic
        }
    }

    /// Fast CSV line tokenizer supporting quotes and commas.
    #[must_use]
    pub fn parse_csv_line(line: &str, delimiter: char) -> Vec<CompactString> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '"' {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else if c == delimiter && !in_quotes {
                fields.push(CompactString::from(current.trim()));
                current.clear();
            } else {
                current.push(c);
            }
        }

        fields.push(CompactString::from(current.trim()));
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parser_with_quotes() {
        let line = r#"SKU-001,"High Torque Motor, 24V",149.99,"In Stock""#;
        let fields = FormatDecoder::parse_csv_line(line, ',');
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "SKU-001");
        assert_eq!(fields[1], "High Torque Motor, 24V");
        assert_eq!(fields[2], "149.99");
        assert_eq!(fields[3], "In Stock");
    }

    #[test]
    fn test_encoding_detection() {
        let utf8_bytes = "کد کالا,نام کالا".as_bytes();
        assert_eq!(FormatDecoder::detect_encoding(utf8_bytes), CharacterEncoding::Utf8);
    }
}
