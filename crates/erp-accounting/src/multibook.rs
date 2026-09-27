//! Parallel Multi-Book Accounting (Local Statutory GAAP, IFRS / Group, Analytical Management).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountingBook {
    LocalStatutoryGaap,
    IfrsGroupConsolidation,
    AnalyticalManagement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBookJournalLine {
    pub account: String,
    pub debit: Decimal,
    pub credit: Decimal,
    pub cost_center: Option<String>,
    pub project_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBookTransaction {
    pub transaction_id: String,
    pub posting_date: chrono::NaiveDate,
    pub company: String,
    pub book: AccountingBook,
    pub lines: Vec<MultiBookJournalLine>,
    pub memo: String,
}

impl MultiBookTransaction {
    pub fn is_balanced(&self) -> bool {
        let total_debit: Decimal = self.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = self.lines.iter().map(|l| l.credit).sum();
        total_debit == total_credit
    }
}

pub struct MultiBookPostingEngine;

impl MultiBookPostingEngine {
    /// Concurrently generates balanced journal entries across all target accounting books.
    pub fn dispatch_parallel_entry(
        tx_id: &str,
        date: chrono::NaiveDate,
        company: &str,
        base_lines: &[MultiBookJournalLine],
        memo: &str,
    ) -> Result<Vec<MultiBookTransaction>, String> {
        let total_debit: Decimal = base_lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = base_lines.iter().map(|l| l.credit).sum();
        if total_debit != total_credit {
            return Err("Cannot dispatch unbalanced parallel transaction".into());
        }

        let books = [
            AccountingBook::LocalStatutoryGaap,
            AccountingBook::IfrsGroupConsolidation,
            AccountingBook::AnalyticalManagement,
        ];

        let txs = books
            .iter()
            .map(|book| MultiBookTransaction {
                transaction_id: format!("{tx_id}_{:?}", book),
                posting_date: date,
                company: company.to_string(),
                book: book.clone(),
                lines: base_lines.to_vec(),
                memo: memo.to_string(),
            })
            .collect();

        Ok(txs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parallel_multibook_dispatch() {
        let lines = vec![
            MultiBookJournalLine {
                account: "1110-Bank".into(),
                debit: dec!(1000.00),
                credit: dec!(0.00),
                cost_center: Some("CC-HQ".into()),
                project_code: None,
            },
            MultiBookJournalLine {
                account: "4100-Revenue".into(),
                debit: dec!(0.00),
                credit: dec!(1000.00),
                cost_center: Some("CC-HQ".into()),
                project_code: None,
            },
        ];

        let txs = MultiBookPostingEngine::dispatch_parallel_entry(
            "TX-991",
            chrono::NaiveDate::from_ymd_opt(2026, 9, 27).unwrap(),
            "Acme Global",
            &lines,
            "Customer Payment Receipt",
        ).unwrap();

        assert_eq!(txs.len(), 3);
        assert!(txs.iter().all(MultiBookTransaction::is_balanced));
    }
}
