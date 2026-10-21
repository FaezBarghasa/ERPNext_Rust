//! High-Velocity Universal Data Ingestion Engine (`erp-importer`).
//!
//! Implements Pillar XXI: Memory-bounded (<80MB RSS) streaming tabular ingestion,
//! automated character encoding detection, fuzzy multilingual header classification
//! (English, Farsi, Japanese, Chinese), and failure diff spreadsheet generation.

pub mod diff_reporter;
pub mod format_decoder;
pub mod fuzzy_matcher;
pub mod mariadb_importer;
pub mod streaming_pipeline;

use serde::{Deserialize, Serialize};

pub use diff_reporter::{DiffReporter, ImportExecutionSummary, RowValidationError};
pub use format_decoder::{CharacterEncoding, FormatDecoder, IngestionFormat, RawDataRow};
pub use fuzzy_matcher::{FuzzyMatcher, TargetFieldSchema};
pub use mariadb_importer::{
    MariaDbImportConfig, MariaDbMigrationEngine, MariaDbTableDump, MigrationReconciliationReport,
    ReconciliationDiscrepancy,
};
pub use streaming_pipeline::{StreamingImportConfig, StreamingPipelineProcessor};

/// Column auto-mapping recommendation item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnMappingItem {
    pub source_column: String,
    pub target_field: String,
    pub confidence: f32,
    pub is_required: bool,
}

/// Target export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Status of an asynchronous or streaming import job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Record of an import job execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportJobRecord {
    pub job_id: String,
    pub doctype: String,
    pub status: ImportJobStatus,
    pub total_rows: usize,
    pub imported_rows: usize,
    pub failed_rows: usize,
    pub errors: Vec<RowValidationError>,
    pub created_at: String,
}

/// High-level Unified Data Ingestion Facade.
pub struct UniversalDataImporter;

impl UniversalDataImporter {
    /// Ingests CSV or delimited text using schema definitions and produces JSON documents and execution summaries.
    #[must_use]
    pub fn import_csv(
        csv_content: &str,
        schema: &[TargetFieldSchema],
        dry_run: bool,
    ) -> (Vec<serde_json::Value>, ImportExecutionSummary) {
        let config = StreamingImportConfig {
            chunk_size: 5000,
            max_memory_mb: 80,
            dry_run,
        };
        StreamingPipelineProcessor::process_csv_stream(csv_content, schema, &config)
    }
}

/// Complete Import/Export Studio supporting visual column auto-mapping and error CSV generation.
pub struct ImportExportStudio;

impl ImportExportStudio {
    /// Suggests column mappings by comparing CSV headers against target schema fields.
    #[must_use]
    pub fn suggest_mappings(
        csv_headers: &[&str],
        schema: &[TargetFieldSchema],
    ) -> Vec<ColumnMappingItem> {
        let auto_map = FuzzyMatcher::auto_map_headers(csv_headers, schema);
        let mut mappings = Vec::new();

        for (idx, &header) in csv_headers.iter().enumerate() {
            if let Some(target_name) = auto_map.get(&idx) {
                let target_field = schema.iter().find(|s| s.field_name == *target_name);
                let is_req = target_field.is_some_and(|s| s.is_required);
                mappings.push(ColumnMappingItem {
                    source_column: header.to_string(),
                    target_field: target_name.to_string(),
                    confidence: 0.95,
                    is_required: is_req,
                });
            }
        }
        mappings
    }

    /// Exports rows of document JSON into CSV or JSON bytes.
    pub fn export_dataset(
        rows: &[serde_json::Value],
        fields: &[String],
        format: ExportFormat,
    ) -> Result<Vec<u8>, String> {
        match format {
            ExportFormat::Json => {
                let bytes = serde_json::to_vec_pretty(rows)
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
                Ok(bytes)
            }
            ExportFormat::Csv => {
                let mut csv_str = String::new();
                csv_str.push_str(&fields.join(","));
                csv_str.push('\n');

                for row in rows {
                    let line_vals: Vec<String> = fields
                        .iter()
                        .map(|f| {
                            row.get(f).map_or_else(String::new, |v| match v {
                                serde_json::Value::String(s) => {
                                    if s.contains(',') || s.contains('"') || s.contains('\n') {
                                        format!("\"{}\"", s.replace('"', "\"\""))
                                    } else {
                                        s.clone()
                                    }
                                }
                                serde_json::Value::Null => String::new(),
                                other => other.to_string(),
                            })
                        })
                        .collect();
                    csv_str.push_str(&line_vals.join(","));
                    csv_str.push('\n');
                }
                Ok(csv_str.into_bytes())
            }
        }
    }

    /// Generates error CSV report string from validation errors.
    #[must_use]
    pub fn generate_error_csv(errors: &[RowValidationError]) -> String {
        let mut out = String::from("row_number,field_name,error_message,raw_value\n");
        for err in errors {
            out.push_str(&format!(
                "{},\"{}\",\"{}\",\"{}\"\n",
                err.row_index,
                err.column_name.replace('"', "\"\""),
                err.error_message.replace('"', "\"\""),
                err.raw_value.replace('"', "\"\""),
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_export_studio_auto_map_and_export() {
        let schema = FuzzyMatcher::default_item_schema();
        let headers = ["Item Code", "Item Name", "Standard Rate"];
        let mappings = ImportExportStudio::suggest_mappings(&headers, &schema);

        assert!(mappings.iter().any(|m| m.target_field == "item_code"));
        assert!(mappings.iter().any(|m| m.target_field == "item_name"));

        let rows = vec![
            serde_json::json!({ "item_code": "ITEM-001", "item_name": "Mechanical Keyboard", "rate": 150 }),
            serde_json::json!({ "item_code": "ITEM-002", "item_name": "Ergo Mouse, USB-C", "rate": 75 }),
        ];
        let fields = vec!["item_code".to_string(), "item_name".to_string(), "rate".to_string()];

        let csv_bytes = ImportExportStudio::export_dataset(&rows, &fields, ExportFormat::Csv).unwrap();
        let csv_text = String::from_utf8(csv_bytes).unwrap();
        assert!(csv_text.contains("item_code,item_name,rate"));
        assert!(csv_text.contains("\"Ergo Mouse, USB-C\""));

        let json_bytes = ImportExportStudio::export_dataset(&rows, &fields, ExportFormat::Json).unwrap();
        let json_val: Vec<serde_json::Value> = serde_json::from_slice(&json_bytes).unwrap();
        assert_eq!(json_val.len(), 2);
    }
}
