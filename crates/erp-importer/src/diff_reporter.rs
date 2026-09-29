//! Import Validation Diff & Error Spreadsheet Generator (`erp_importer::diff_reporter`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Detailed cell-level validation error record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowValidationError {
    pub row_index: usize,
    pub column_name: CompactString,
    pub raw_value: CompactString,
    pub error_message: CompactString,
}

/// Ingestion summary metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportExecutionSummary {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub failed_rows: usize,
    pub duration_ms: u64,
    pub errors: Vec<RowValidationError>,
}

/// Generates failure diff annotations for spreadsheet error exports.
pub struct DiffReporter;

impl DiffReporter {
    /// Compiles a CSV diff highlighting erroneous rows with failure reasons.
    #[must_use]
    pub fn generate_error_csv(
        headers: &[&str],
        failed_rows: &[(usize, Vec<CompactString>, Vec<RowValidationError>)],
    ) -> String {
        let mut out = String::new();

        // Add headers + trailing Error Column
        for h in headers {
            out.push_str(h);
            out.push(',');
        }
        out.push_str("__ERROR_REASONS__\n");

        for (_row_idx, fields, errs) in failed_rows {
            for f in fields {
                out.push('"');
                out.push_str(&f.replace('"', "\"\""));
                out.push_str("\",");
            }

            let reasons: Vec<String> = errs
                .iter()
                .map(|e| format!("[{}: {}]", e.column_name, e.error_message))
                .collect();

            out.push('"');
            out.push_str(&reasons.join("; ").replace('"', "\"\""));
            out.push_str("\"\n");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_csv_generation() {
        let headers = ["item_code", "item_name", "price"];
        let failed_row = (
            2usize,
            vec!["ITEM-01".into(), "Widget".into(), "invalid_number".into()],
            vec![RowValidationError {
                row_index: 2,
                column_name: "price".into(),
                raw_value: "invalid_number".into(),
                error_message: "Must be a valid decimal number".into(),
            }],
        );

        let csv = DiffReporter::generate_error_csv(&headers, &[failed_row]);
        assert!(csv.contains("__ERROR_REASONS__"));
        assert!(csv.contains("[price: Must be a valid decimal number]"));
    }
}
