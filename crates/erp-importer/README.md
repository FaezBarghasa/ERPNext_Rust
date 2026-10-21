# 📥 erp-importer

> **Universal Streaming Data Ingestion, Fuzzy Schema Auto-Mapping & Migration Engine for `rust-web-land` (`rwl`)**

`erp-importer` delivers memory-bounded (<80MB RSS) streaming tabular ingestion, automated character encoding detection, fuzzy multilingual column classification (English, Farsi, Japanese, Chinese), and granular failure diff spreadsheet generation.

---

## ⚡ Core Capabilities

- **Streaming Tabular Pipeline (`streaming_pipeline.rs`)**:
  - Memory-bounded streaming CSV, TSV, and delimited row ingestion without loading full payloads into memory.
  - Chunked validation, batched record insertion, and error-resilient rollback boundaries.
- **Multilingual Fuzzy Header Matching (`fuzzy_matcher.rs`)**:
  - High-confidence automatic column mapping across multiple natural languages (English, Persian/Farsi, Simplified/Traditional Chinese, Japanese).
  - Levenshtein and token-set similarity algorithms scoring target DocType schema fields.
- **Character Encoding & Format Decoder (`format_decoder.rs`)**:
  - Automatic detection of UTF-8, UTF-16LE/BE, Windows-1252, and ISO-8859 variants.
  - Universal format normalization into typed `RawDataRow` primitives.
- **Failure Diff Reporting (`diff_reporter.rs`)**:
  - Generates downloadable discrepancy workbooks highlighting rejected rows and specific field validation errors.
- **MariaDB Migration Engine (`mariadb_importer.rs`)**:
  - Native table dump parsing and automated schema reconciliation report generator.

---

## 📦 Workspace Architecture

```
crates/erp-importer/
├── Cargo.toml
└── src/
    ├── diff_reporter.rs        # Error reporting & diff workbook generator
    ├── format_decoder.rs       # Ingestion decoding & encoding detector
    ├── fuzzy_matcher.rs        # Multilingual header classification
    ├── lib.rs                  # Public exports & pipeline contracts
    ├── mariadb_importer.rs     # Legacy MariaDB migration reconciler
    └── streaming_pipeline.rs   # Memory-bounded streaming processor
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** ecosystem.
