//! High-Velocity Universal Data Ingestion Engine (`erp-importer`).
//!
//! Implements Pillar XXI: Memory-bounded (<80MB RSS) streaming tabular ingestion,
//! automated character encoding detection, fuzzy multilingual header classification
//! (English, Farsi, Japanese, Chinese), and failure diff spreadsheet generation.

pub mod diff_reporter;
pub mod format_decoder;
pub mod fuzzy_matcher;
pub mod streaming_pipeline;

pub use diff_reporter::{DiffReporter, ImportExecutionSummary, RowValidationError};
pub use format_decoder::{CharacterEncoding, FormatDecoder, IngestionFormat, RawDataRow};
pub use fuzzy_matcher::{FuzzyMatcher, TargetFieldSchema};
pub use streaming_pipeline::{StreamingImportConfig, StreamingPipelineProcessor};

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
