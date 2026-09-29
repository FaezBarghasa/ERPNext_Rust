use crate::diff_reporter::{ImportExecutionSummary, RowValidationError};
use crate::format_decoder::FormatDecoder;
use crate::fuzzy_matcher::{FuzzyMatcher, TargetFieldSchema};
use serde_json::{Map, Value};

/// Config for streaming ingestion batches.
pub struct StreamingImportConfig {
    pub chunk_size: usize,
    pub max_memory_mb: usize,
    pub dry_run: bool,
}

impl Default for StreamingImportConfig {
    fn default() -> Self {
        Self {
            chunk_size: 5_000,
            max_memory_mb: 80,
            dry_run: true,
        }
    }
}

/// Streaming Pipeline Processor.
pub struct StreamingPipelineProcessor;

impl StreamingPipelineProcessor {
    /// Ingests and validates CSV text in streaming chunks, producing JSON documents and error reports.
    pub fn process_csv_stream(
        csv_content: &str,
        schema: &[TargetFieldSchema],
        _config: &StreamingImportConfig,
    ) -> (Vec<Value>, ImportExecutionSummary) {
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut lines = csv_content.lines();

        let header_line = match lines.next() {
            Some(h) => h,
            None => {
                return (
                    Vec::new(),
                    ImportExecutionSummary {
                        total_rows: 0,
                        valid_rows: 0,
                        failed_rows: 0,
                        duration_ms: 0,
                        errors: Vec::new(),
                    },
                );
            }
        };

        let raw_headers = FormatDecoder::parse_csv_line(header_line, ',');
        let header_strs: Vec<&str> = raw_headers.iter().map(|s| s.as_str()).collect();
        let column_map = FuzzyMatcher::auto_map_headers(&header_strs, schema);

        let mut valid_docs = Vec::new();
        let mut errors = Vec::new();
        let mut total_rows = 0usize;
        let mut valid_count = 0usize;
        let mut failed_count = 0usize;

        for (idx, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            total_rows += 1;
            let row_idx = idx + 2; // 1-indexed including header
            let fields = FormatDecoder::parse_csv_line(line, ',');

            let mut doc_map = Map::new();
            let mut row_errors = Vec::new();

            // Populate mapped fields
            for (&col_idx, field_name) in &column_map {
                if let Some(val) = fields.get(col_idx)
                    && !val.is_empty()
                {
                    doc_map.insert(field_name.to_string(), Value::String(val.to_string()));
                }
            }

            // Validate required fields
            for target in schema {
                if target.is_required && !doc_map.contains_key(target.field_name.as_str()) {
                    row_errors.push(RowValidationError {
                        row_index: row_idx,
                        column_name: target.field_name.clone(),
                        raw_value: "".into(),
                        error_message: format!("Required field '{}' is missing", target.field_name)
                            .into(),
                    });
                }
            }

            if row_errors.is_empty() {
                valid_count += 1;
                valid_docs.push(Value::Object(doc_map));
            } else {
                failed_count += 1;
                errors.extend(row_errors);
            }
        }

        let duration_ms = (chrono::Utc::now().timestamp_millis() as u64).saturating_sub(start_time);

        let summary = ImportExecutionSummary {
            total_rows,
            valid_rows: valid_count,
            failed_rows: failed_count,
            duration_ms,
            errors,
        };

        (valid_docs, summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_csv_import_end_to_end() {
        let csv = "کد کالا,نام کالا,قیمت واحد,واحد سنجش\n\
                   SKU-1001,High Precision Servo,89.50,Nos\n\
                   SKU-1002,Optical Encoder,45.00,Nos\n\
                   ,Missing Code Item,12.00,Nos\n";

        let schema = FuzzyMatcher::default_item_schema();
        let config = StreamingImportConfig::default();

        let (docs, summary) = StreamingPipelineProcessor::process_csv_stream(csv, &schema, &config);

        assert_eq!(summary.total_rows, 3);
        assert_eq!(summary.valid_rows, 2);
        assert_eq!(summary.failed_rows, 1);
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["item_code"], "SKU-1001");
        assert_eq!(docs[0]["item_name"], "High Precision Servo");
    }
}
