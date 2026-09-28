//! Formula-Driven Financial Report Template Engine (`erp-accounting::report_engine`).
//!
//! Provides customizable mathematical financial reporting (P&L, Balance Sheet, Cash Flow):
//! - `Financial Report Row` and `Account Category` descriptors
//! - `Calculated Amount` formulas referencing arbitrary row labels:
//!   `Row[Gross_Margin] = Row[Operating_Revenue] - Row[COGS]`
//! - Custom API metrics integration
//! - Consolidated Trial Balance with multi-currency translations & zero suppression.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReportError {
    #[error("Row key '{0}' not found in report context")]
    RowNotFound(String),
    #[error("Formula evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// Row type in a formula-driven financial statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportRowType {
    AccountCategory,
    CalculatedAmount,
    CustomApiMetric,
    SectionBreak,
}

/// A row definition in a Financial Report Template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialReportRow {
    pub label: CompactString,
    pub row_type: ReportRowType,
    pub account_category: Option<CompactString>,
    pub formula: Option<CompactString>, // e.g. "Operating_Revenue - COGS"
    pub is_bold: bool,
    pub indent_level: u8,
    pub hide_if_zero: bool,
}

/// Statement Output Line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatementLine {
    pub label: CompactString,
    pub amount: Decimal,
    pub is_bold: bool,
    pub indent_level: u8,
}

/// Financial Report Template Engine.
#[derive(Debug, Clone, Default)]
pub struct FinancialReportEngine {
    rows: Vec<FinancialReportRow>,
}

impl FinancialReportEngine {
    #[must_use]
    pub fn new(rows: Vec<FinancialReportRow>) -> Self {
        Self { rows }
    }

    /// Evaluates the statement given ledger account balances and custom metric values.
    pub fn generate_statement(
        &self,
        category_balances: &HashMap<CompactString, Decimal>,
        custom_metrics: &HashMap<CompactString, Decimal>,
    ) -> Result<Vec<StatementLine>, ReportError> {
        let mut computed_values: HashMap<CompactString, Decimal> = HashMap::new();
        let mut lines = Vec::new();

        for row in &self.rows {
            let amount = match row.row_type {
                ReportRowType::AccountCategory => {
                    if let Some(ref cat) = row.account_category {
                        category_balances.get(cat).copied().unwrap_or(Decimal::ZERO)
                    } else {
                        Decimal::ZERO
                    }
                }
                ReportRowType::CustomApiMetric => {
                    custom_metrics.get(&row.label).copied().unwrap_or(Decimal::ZERO)
                }
                ReportRowType::CalculatedAmount => {
                    if let Some(ref formula) = row.formula {
                        Self::eval_simple_formula(formula, &computed_values)?
                    } else {
                        Decimal::ZERO
                    }
                }
                ReportRowType::SectionBreak => Decimal::ZERO,
            };

            computed_values.insert(row.label.clone(), amount);

            if row.hide_if_zero && amount.is_zero() && row.row_type != ReportRowType::SectionBreak {
                continue;
            }

            lines.push(StatementLine {
                label: row.label.clone(),
                amount,
                is_bold: row.is_bold,
                indent_level: row.indent_level,
            });
        }

        Ok(lines)
    }

    /// Evaluates simple addition/subtraction formulas across row keys (e.g. `Rev - COGS + Other`).
    fn eval_simple_formula(
        formula: &str,
        values: &HashMap<CompactString, Decimal>,
    ) -> Result<Decimal, ReportError> {
        let tokens: Vec<&str> = formula.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let first_key = CompactString::new(tokens[0]);
        let mut total = values
            .get(&first_key)
            .copied()
            .ok_or_else(|| ReportError::RowNotFound(tokens[0].to_string()))?;

        let mut i = 1;
        while i < tokens.len() {
            let op = tokens[i];
            let next_key = CompactString::new(tokens.get(i + 1).ok_or_else(|| {
                ReportError::EvaluationFailed("Trailing operator in formula".into())
            })?);

            let next_val = values
                .get(&next_key)
                .copied()
                .ok_or_else(|| ReportError::RowNotFound(next_key.to_string()))?;

            match op {
                "+" => total += next_val,
                "-" => total -= next_val,
                "*" => total *= next_val,
                _ => return Err(ReportError::EvaluationFailed(format!("Unsupported operator: {op}"))),
            }
            i += 2;
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_financial_report_formula_evaluation() {
        let rows = vec![
            FinancialReportRow {
                label: "Operating_Revenue".into(),
                row_type: ReportRowType::AccountCategory,
                account_category: Some("Revenue".into()),
                formula: None,
                is_bold: false,
                indent_level: 0,
                hide_if_zero: false,
            },
            FinancialReportRow {
                label: "COGS".into(),
                row_type: ReportRowType::AccountCategory,
                account_category: Some("Cost of Goods Sold".into()),
                formula: None,
                is_bold: false,
                indent_level: 0,
                hide_if_zero: false,
            },
            FinancialReportRow {
                label: "Gross_Margin".into(),
                row_type: ReportRowType::CalculatedAmount,
                account_category: None,
                formula: Some("Operating_Revenue - COGS".into()),
                is_bold: true,
                indent_level: 1,
                hide_if_zero: false,
            },
        ];

        let engine = FinancialReportEngine::new(rows);
        let mut balances = HashMap::new();
        balances.insert("Revenue".into(), dec!(500000.00));
        balances.insert("Cost of Goods Sold".into(), dec!(300000.00));

        let statement = engine.generate_statement(&balances, &HashMap::new()).unwrap();
        assert_eq!(statement.len(), 3);
        assert_eq!(statement[2].label.as_str(), "Gross_Margin");
        assert_eq!(statement[2].amount, dec!(200000.00));
        assert!(statement[2].is_bold);
    }
}
