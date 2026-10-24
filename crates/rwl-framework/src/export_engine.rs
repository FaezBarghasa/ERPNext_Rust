//! Universal Document Export & Data Serialization Engine (`frappe-framework::export_engine`).
//!
//! Provides enterprise export capabilities matching ERPNext, Odoo, and emdash:
//! - RFC 4180 compliant CSV export with proper quotation and escaping
//! - TSV and Tabular Excel-compatible HTML/XML data streaming
//! - JSON and JSON Lines (JSONL) high-throughput bulk export
//! - Selective column mapping, localized header labeling, and numeric formatting

use serde::{Deserialize, Serialize};
use std::io::Write;
use thiserror::Error;

/// Export generation errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExportError {
    #[error("IO error during export: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("No columns specified for tabular export")]
    NoColumnsSpecified,
}

/// Target export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Tsv,
    Json,
    JsonLines,
    HtmlTable,
}

/// Column definition for tabular export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportColumn {
    pub fieldname: String,
    pub header_label: String,
}

impl ExportColumn {
    #[must_use]
    pub fn new(fieldname: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            fieldname: fieldname.into(),
            header_label: label.into(),
        }
    }
}

/// Universal Document Exporter.
pub struct DocumentExporter;

impl DocumentExporter {
    /// Serializes a dataset of JSON objects to the chosen export format.
    pub fn export_to_string(
        format: ExportFormat,
        columns: &[ExportColumn],
        rows: &[serde_json::Value],
    ) -> Result<String, ExportError> {
        let mut buffer = Vec::new();
        Self::export_to_writer(format, columns, rows, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| ExportError::Serialization(e.to_string()))
    }

    /// Streams exported data directly into any `std::io::Write` target.
    pub fn export_to_writer<W: Write>(
        format: ExportFormat,
        columns: &[ExportColumn],
        rows: &[serde_json::Value],
        writer: &mut W,
    ) -> Result<(), ExportError> {
        match format {
            ExportFormat::Csv => Self::write_delimited(columns, rows, b',', writer),
            ExportFormat::Tsv => Self::write_delimited(columns, rows, b'\t', writer),
            ExportFormat::Json => serde_json::to_writer_pretty(writer, rows)
                .map_err(|e| ExportError::Serialization(e.to_string())),
            ExportFormat::JsonLines => {
                for row in rows {
                    serde_json::to_writer(&mut *writer, row)
                        .map_err(|e| ExportError::Serialization(e.to_string()))?;
                    writer
                        .write_all(b"\n")
                        .map_err(|e| ExportError::Io(e.to_string()))?;
                }
                Ok(())
            }
            ExportFormat::HtmlTable => Self::write_html_table(columns, rows, writer),
        }
    }

    fn write_delimited<W: Write>(
        columns: &[ExportColumn],
        rows: &[serde_json::Value],
        delimiter: u8,
        writer: &mut W,
    ) -> Result<(), ExportError> {
        if columns.is_empty() {
            return Err(ExportError::NoColumnsSpecified);
        }

        let delim_char = delimiter as char;

        // Write Header Row
        let header_line = columns
            .iter()
            .map(|c| Self::escape_field(&c.header_label, delim_char))
            .collect::<Vec<String>>()
            .join(&delim_char.to_string());

        writer
            .write_all(header_line.as_bytes())
            .map_err(|e| ExportError::Io(e.to_string()))?;
        writer
            .write_all(b"\r\n")
            .map_err(|e| ExportError::Io(e.to_string()))?;

        // Write Data Rows
        for row in rows {
            let row_line = columns
                .iter()
                .map(|col| {
                    let val_str = match row.get(&col.fieldname) {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::Bool(b)) => b.to_string(),
                        Some(serde_json::Value::Null) | None => String::new(),
                        Some(other) => other.to_string(),
                    };
                    Self::escape_field(&val_str, delim_char)
                })
                .collect::<Vec<String>>()
                .join(&delim_char.to_string());

            writer
                .write_all(row_line.as_bytes())
                .map_err(|e| ExportError::Io(e.to_string()))?;
            writer
                .write_all(b"\r\n")
                .map_err(|e| ExportError::Io(e.to_string()))?;
        }

        Ok(())
    }

    fn escape_field(text: &str, delimiter: char) -> String {
        let needs_quote = text.contains(delimiter)
            || text.contains('"')
            || text.contains('\n')
            || text.contains('\r');

        if needs_quote {
            let escaped = text.replace('"', "\"\"");
            format!("\"{escaped}\"")
        } else {
            text.to_string()
        }
    }

    fn write_html_table<W: Write>(
        columns: &[ExportColumn],
        rows: &[serde_json::Value],
        writer: &mut W,
    ) -> Result<(), ExportError> {
        let mut html = String::from(
            "<table border=\"1\" cellspacing=\"0\" cellpadding=\"4\">\n  <thead>\n    <tr>\n",
        );
        for col in columns {
            html.push_str(&format!(
                "      <th>{}</th>\n",
                html_escape(&col.header_label)
            ));
        }
        html.push_str("    </tr>\n  </thead>\n  <tbody>\n");

        for row in rows {
            html.push_str("    <tr>\n");
            for col in columns {
                let cell_val = match row.get(&col.fieldname) {
                    Some(serde_json::Value::String(s)) => s.as_str(),
                    Some(serde_json::Value::Number(n)) => &n.to_string(),
                    Some(serde_json::Value::Bool(b)) => {
                        if *b {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    Some(serde_json::Value::Null) | None => "",
                    _ => "",
                };
                html.push_str(&format!("      <td>{}</td>\n", html_escape(cell_val)));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n</table>");

        writer
            .write_all(html.as_bytes())
            .map_err(|e| ExportError::Io(e.to_string()))
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc4180_csv_export_with_quotes_and_commas() {
        let columns = vec![
            ExportColumn::new("name", "Invoice #"),
            ExportColumn::new("customer", "Customer Name"),
            ExportColumn::new("amount", "Total ($)"),
        ];

        let rows = vec![
            serde_json::json!({
                "name": "ACC-INV-001",
                "customer": "Acme, Inc.", // Contains comma -> must quote
                "amount": 1250.50
            }),
            serde_json::json!({
                "name": "ACC-INV-002",
                "customer": "John \"The Boss\" Smith", // Contains quote -> must escape
                "amount": 3400.00
            }),
        ];

        let csv = DocumentExporter::export_to_string(ExportFormat::Csv, &columns, &rows).unwrap();
        let lines: Vec<&str> = csv.split("\r\n").filter(|l| !l.is_empty()).collect();

        assert_eq!(lines[0], "Invoice #,Customer Name,Total ($)");
        assert_eq!(lines[1], "ACC-INV-001,\"Acme, Inc.\",1250.5");
        assert_eq!(
            lines[2],
            "ACC-INV-002,\"John \"\"The Boss\"\" Smith\",3400.0"
        );
    }

    #[test]
    fn test_jsonlines_and_html_table_export() {
        let columns = vec![
            ExportColumn::new("id", "ID"),
            ExportColumn::new("status", "Status"),
        ];
        let rows = vec![
            serde_json::json!({"id": "TASK-1", "status": "Open"}),
            serde_json::json!({"id": "TASK-2", "status": "Completed"}),
        ];

        let jsonl =
            DocumentExporter::export_to_string(ExportFormat::JsonLines, &columns, &rows).unwrap();
        let jsonl_lines: Vec<&str> = jsonl.lines().collect();
        assert_eq!(jsonl_lines.len(), 2);

        let html =
            DocumentExporter::export_to_string(ExportFormat::HtmlTable, &columns, &rows).unwrap();
        assert!(html.contains("<th>ID</th>"));
        assert!(html.contains("<td>TASK-1</td>"));
    }
}
