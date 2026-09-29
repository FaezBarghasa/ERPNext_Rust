//! Advanced Reporting, Analytics & Dynamic Pivot KPI Engine (`frappe-framework::report_engine`).
//!
//! Provides report building capabilities matching ERPNext Script Reports and Odoo Pivot Views:
//! - Multi-column tabular reports with typed schemas (Currency, Int, Float, Percent, Date, String)
//! - Summary KPI metric cards with change indicators
//! - Dynamic pivot table aggregations (Group By rows, Pivot on column, Sum/Count/Avg/Min/Max)
//! - Time-series chart datasets generation

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

/// Report engine errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReportError {
    #[error("Column '{0}' not found in report dataset")]
    ColumnNotFound(String),
    #[error("Invalid aggregation function '{0}'")]
    InvalidAggregation(String),
}

/// Report column schema definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportColumn {
    pub fieldname: String,
    pub label: String,
    pub fieldtype: String, // "Currency", "Int", "Float", "Percent", "Date", "Data"
    pub width: u16,
}

impl ReportColumn {
    #[must_use]
    pub fn new(
        fieldname: impl Into<String>,
        label: impl Into<String>,
        fieldtype: impl Into<String>,
        width: u16,
    ) -> Self {
        Self {
            fieldname: fieldname.into(),
            label: label.into(),
            fieldtype: fieldtype.into(),
            width,
        }
    }
}

/// Summary KPI metric card displayed at the top of a report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummaryCard {
    pub label: String,
    pub value_formatted: String,
    pub raw_numeric: f64,
    pub indicator_color: String, // "green", "red", "blue", "orange"
    pub subtext: Option<String>,
}

/// Dynamic Pivot Table Aggregation Kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PivotAggregate {
    Sum,
    Count,
    Average,
    Min,
    Max,
}

/// Dynamic Pivot Table Result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotTableResult {
    pub row_keys: Vec<String>,
    pub col_keys: Vec<String>,
    /// Matrix of values indexed by (row_key, col_key) -> aggregated value.
    pub matrix: HashMap<String, HashMap<String, f64>>,
    pub row_totals: HashMap<String, f64>,
    pub col_totals: HashMap<String, f64>,
    pub grand_total: f64,
}

/// Complete Report Execution Result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub columns: Vec<ReportColumn>,
    pub rows: Vec<serde_json::Value>,
    pub summary_cards: Vec<ReportSummaryCard>,
    pub total_count: usize,
}

/// Report Calculation and Pivot Engine.
pub struct ReportEngine;

impl ReportEngine {
    /// Calculates a summary metric card summing a numeric field across all rows.
    #[must_use]
    pub fn calculate_sum_kpi(
        label: impl Into<String>,
        fieldname: &str,
        rows: &[serde_json::Value],
        currency_symbol: Option<&str>,
        color: &str,
    ) -> ReportSummaryCard {
        let mut total: f64 = 0.0;
        for row in rows {
            if let Some(num) = row.get(fieldname).and_then(|v| v.as_f64()) {
                total += num;
            }
        }

        let symbol = currency_symbol.unwrap_or("");
        let formatted = if symbol.is_empty() {
            format!("{total:.2}")
        } else {
            format!("{symbol}{total:.2}")
        };

        ReportSummaryCard {
            label: label.into(),
            value_formatted: formatted,
            raw_numeric: total,
            indicator_color: color.to_string(),
            subtext: Some(format!("Across {} records", rows.len())),
        }
    }

    /// Computes a multi-dimensional pivot table from flat JSON records.
    pub fn compute_pivot(
        rows: &[serde_json::Value],
        row_field: &str,
        col_field: &str,
        val_field: &str,
        aggregate: PivotAggregate,
    ) -> PivotTableResult {
        // Collect unique ordered row and col keys
        let mut row_key_set = BTreeMap::new();
        let mut col_key_set = BTreeMap::new();

        // (row_key, col_key) -> Vec<f64>
        let mut bucket: HashMap<(String, String), Vec<f64>> = HashMap::new();

        for r in rows {
            let row_val = r
                .get(row_field)
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let col_val = r
                .get(col_field)
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let num = r.get(val_field).and_then(|v| v.as_f64()).unwrap_or(0.0);

            row_key_set.insert(row_val.clone(), ());
            col_key_set.insert(col_val.clone(), ());

            bucket.entry((row_val, col_val)).or_default().push(num);
        }

        let row_keys: Vec<String> = row_key_set.into_keys().collect();
        let col_keys: Vec<String> = col_key_set.into_keys().collect();

        let mut matrix: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut row_totals: HashMap<String, f64> = HashMap::new();
        let mut col_totals: HashMap<String, f64> = HashMap::new();
        let mut grand_total: f64 = 0.0;

        for r_k in &row_keys {
            for c_k in &col_keys {
                let key = (r_k.clone(), c_k.clone());
                let cell_val = match bucket.get(&key) {
                    Some(values) if !values.is_empty() => match aggregate {
                        PivotAggregate::Sum => values.iter().sum::<f64>(),
                        PivotAggregate::Count => values.len() as f64,
                        PivotAggregate::Average => values.iter().sum::<f64>() / values.len() as f64,
                        PivotAggregate::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
                        PivotAggregate::Max => {
                            values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                        }
                    },
                    _ => 0.0,
                };

                matrix
                    .entry(r_k.clone())
                    .or_default()
                    .insert(c_k.clone(), cell_val);

                *row_totals.entry(r_k.clone()).or_insert(0.0) += cell_val;
                *col_totals.entry(c_k.clone()).or_insert(0.0) += cell_val;
                grand_total += cell_val;
            }
        }

        PivotTableResult {
            row_keys,
            col_keys,
            matrix,
            row_totals,
            col_totals,
            grand_total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_kpi_calculation() {
        let rows = vec![
            serde_json::json!({"customer": "Acme", "amount": 1000.0}),
            serde_json::json!({"customer": "Globex", "amount": 2500.0}),
            serde_json::json!({"customer": "Initech", "amount": 1500.0}),
        ];

        let kpi =
            ReportEngine::calculate_sum_kpi("Total Sales", "amount", &rows, Some("$"), "green");
        assert_eq!(kpi.raw_numeric, 5000.0);
        assert_eq!(kpi.value_formatted, "$5000.00");
    }

    #[test]
    fn test_dynamic_pivot_table_generation() {
        let rows = vec![
            serde_json::json!({"region": "North", "quarter": "Q1", "revenue": 100.0}),
            serde_json::json!({"region": "North", "quarter": "Q2", "revenue": 150.0}),
            serde_json::json!({"region": "South", "quarter": "Q1", "revenue": 200.0}),
            serde_json::json!({"region": "South", "quarter": "Q2", "revenue": 250.0}),
        ];

        let pivot =
            ReportEngine::compute_pivot(&rows, "region", "quarter", "revenue", PivotAggregate::Sum);

        assert_eq!(pivot.row_keys, vec!["North", "South"]);
        assert_eq!(pivot.col_keys, vec!["Q1", "Q2"]);
        assert_eq!(pivot.matrix["North"]["Q1"], 100.0);
        assert_eq!(pivot.matrix["North"]["Q2"], 150.0);
        assert_eq!(pivot.matrix["South"]["Q1"], 200.0);
        assert_eq!(pivot.matrix["South"]["Q2"], 250.0);
        assert_eq!(pivot.row_totals["North"], 250.0);
        assert_eq!(pivot.row_totals["South"], 450.0);
        assert_eq!(pivot.grand_total, 700.0);
    }
}
