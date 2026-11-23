//! Live MariaDB Direct Streaming Migration Engine & Type Normalizer (`erp-importer::live_streamer`).
//!
//! Provides chunked keyset streaming ingest, metadata parsing from `tabDocType` / `information_schema`,
//! relational child row reconstruction (`tabSales Invoice Item` -> `items: [...]`), and MySQL type normalization:
//! - `datetime(6)` / `datetime` -> RFC3339 UTC strings
//! - `decimal(21,9)` / numeric -> exact `rust_decimal::Decimal`
//! - `tinyint(1)` -> `bool`
//! - `longtext` / `varchar` -> `String`

use chrono::{NaiveDateTime, TimeZone, Utc};
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// MySQL / MariaDB Column Type Classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MariaDbColumnType {
    Varchar,
    LongText,
    TinyIntBool,
    Integer,
    BigInt,
    DecimalVal,
    DateTimeUtc,
    DateOnly,
    JsonVal,
}

/// Metadata description of a MariaDB table column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MariaDbColumnMeta {
    pub column_name: CompactString,
    pub data_type: MariaDbColumnType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

/// Specification of a MariaDB table and its DocType relationship.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MariaDbTableSchema {
    pub table_name: CompactString,
    pub doctype_name: CompactString,
    pub is_child_table: bool,
    pub columns: Vec<MariaDbColumnMeta>,
    pub primary_key_column: CompactString,
}

/// Keyset Cursor Batch for stream processing large tables without memory exhaustion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeysetBatch<T> {
    pub records: Vec<T>,
    pub last_cursor: Option<CompactString>,
    pub has_more: bool,
    pub total_processed: usize,
}

/// Direct Streaming Source Trait allowing pluggable drivers (Mock, TCP, Raw Wire, MySQL/MariaDB).
pub trait MariaDbDirectSource: Send + Sync {
    /// Discovers tables and schemas matching `tab%`.
    fn discover_schemas(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<MariaDbTableSchema>, String>> + Send;

    /// Fetches chunked keyset cursor batches of rows.
    fn fetch_keyset_batch(
        &self,
        table: &str,
        pk: &str,
        cursor: Option<&str>,
        batch_size: usize,
    ) -> impl std::future::Future<Output = Result<KeysetBatch<Map<String, Value>>, String>> + Send;
}

/// Live MariaDB Streaming Migration Engine.
pub struct LiveMariaDbStreamer;

impl LiveMariaDbStreamer {
    /// Normalizes a single MySQL/MariaDB field value into its canonical SurrealDB/JSON counterpart.
    pub fn normalize_mysql_value(col_type: MariaDbColumnType, raw_val: &Value) -> Value {
        match raw_val {
            Value::Null => Value::Null,
            Value::String(s) => match col_type {
                MariaDbColumnType::DateTimeUtc => {
                    // Try parsing MySQL datetime formats: "2026-10-16 12:00:00" or "2026-10-16 12:00:00.123456"
                    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                    {
                        Value::String(Utc.from_utc_datetime(&naive).to_rfc3339())
                    } else {
                        Value::String(s.clone())
                    }
                }
                MariaDbColumnType::DecimalVal => {
                    if let Ok(dec) = s.parse::<Decimal>() {
                        Value::String(dec.normalize().to_string())
                    } else {
                        Value::String(s.clone())
                    }
                }
                MariaDbColumnType::TinyIntBool => match s.as_str() {
                    "1" | "true" | "TRUE" => Value::Bool(true),
                    "0" | "false" | "FALSE" => Value::Bool(false),
                    _ => Value::String(s.clone()),
                },
                MariaDbColumnType::JsonVal => {
                    serde_json::from_str::<Value>(s).unwrap_or_else(|_| Value::String(s.clone()))
                }
                _ => Value::String(s.clone()),
            },
            Value::Number(n) => match col_type {
                MariaDbColumnType::TinyIntBool => {
                    if let Some(i) = n.as_i64() {
                        Value::Bool(i != 0)
                    } else {
                        Value::Number(n.clone())
                    }
                }
                MariaDbColumnType::DecimalVal => {
                    if let Some(f) = n.as_f64() {
                        if let Ok(d) = Decimal::try_from(f) {
                            Value::String(d.normalize().to_string())
                        } else {
                            Value::Number(n.clone())
                        }
                    } else {
                        Value::Number(n.clone())
                    }
                }
                _ => Value::Number(n.clone()),
            },
            Value::Bool(b) => Value::Bool(*b),
            _ => raw_val.clone(),
        }
    }

    /// Normalizes an entire row according to the table schema columns.
    pub fn normalize_row(schema: &MariaDbTableSchema, mut row: Map<String, Value>) -> Map<String, Value> {
        let col_map: HashMap<&str, MariaDbColumnType> = schema
            .columns
            .iter()
            .map(|c| (c.column_name.as_str(), c.data_type))
            .collect();

        for (k, v) in row.iter_mut() {
            if let Some(&col_type) = col_map.get(k.as_str()) {
                *v = Self::normalize_mysql_value(col_type, v);
            }
        }
        row
    }

    /// Reconstructs parent-child hierarchy by nesting child rows into parent `items: [...]`.
    pub fn nest_child_records(
        parent_docs: &mut [Map<String, Value>],
        child_rows: &[Map<String, Value>],
        parent_doctype: &str,
    ) {
        let mut children_by_parent: HashMap<String, HashMap<String, Vec<Map<String, Value>>>> =
            HashMap::new();

        for child in child_rows {
            let p_type = child.get("parenttype").and_then(|v| v.as_str()).unwrap_or("");
            let p_name = child.get("parent").and_then(|v| v.as_str()).unwrap_or("");
            let p_field = child.get("parentfield").and_then(|v| v.as_str()).unwrap_or("items");

            if p_type == parent_doctype && !p_name.is_empty() {
                children_by_parent
                    .entry(p_name.to_string())
                    .or_default()
                    .entry(p_field.to_string())
                    .or_default()
                    .push(child.clone());
            }
        }

        for parent in parent_docs.iter_mut() {
            if let Some(name) = parent.get("name").and_then(|v| v.as_str())
                && let Some(field_map) = children_by_parent.get_mut(name)
            {
                for (field, items) in field_map {
                    // Sort child rows by idx
                    items.sort_by_key(|c| c.get("idx").and_then(|v| v.as_i64()).unwrap_or(0));
                    let arr: Vec<Value> = items.drain(..).map(Value::Object).collect();
                    parent.insert(field.clone(), Value::Array(arr));
                }
            }
        }
    }
}
