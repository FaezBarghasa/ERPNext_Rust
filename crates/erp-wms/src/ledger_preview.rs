use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of stock entry movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockEntryPurpose {
    /// Issue materials out of warehouse (e.g. scrap, consumption).
    MaterialIssue,
    /// Inward material receipt without purchase order.
    MaterialReceipt,
    /// Transfer between warehouses.
    MaterialTransfer,
    /// Manufacture finished goods from raw materials.
    Manufacture,
    /// Repack / split items into other items.
    Repack,
}

/// Simulated Stock Ledger Entry preview line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulatedSle {
    /// Item code.
    pub item_code: String,
    /// Warehouse.
    pub warehouse: String,
    /// Stock quantity delta (+ for inward, - for outward).
    pub actual_qty: Decimal,
    /// Unit valuation rate.
    pub valuation_rate: Decimal,
    /// Total monetary stock value difference.
    pub stock_value_difference: Decimal,
}

/// Simulated General Ledger Entry preview line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulatedGl {
    /// Account head.
    pub account: String,
    /// Debit amount in company currency.
    pub debit: Decimal,
    /// Credit amount in company currency.
    pub credit: Decimal,
}

/// Comprehensive Ledger Preview response for stock entries before actual submission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockEntryLedgerPreview {
    /// Simulated Stock Ledger Entries.
    pub stock_ledger_entries: Vec<SimulatedSle>,
    /// Simulated General Ledger Entries.
    pub gl_entries: Vec<SimulatedGl>,
    /// Total debits in preview GL.
    pub total_debit: Decimal,
    /// Total credits in preview GL.
    pub total_credit: Decimal,
}

/// Stock Entry line item for ledger simulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockEntryLine {
    /// Item code.
    pub item_code: String,
    /// Source warehouse (if issuing / transferring).
    pub source_warehouse: Option<String>,
    /// Target warehouse (if receiving / transferring).
    pub target_warehouse: Option<String>,
    /// Quantity.
    pub qty: Decimal,
    /// Valuation rate.
    pub basic_rate: Decimal,
    /// Stock in hand account linked to source warehouse.
    pub source_stock_account: Option<String>,
    /// Stock in hand account linked to target warehouse.
    pub target_stock_account: Option<String>,
    /// Expense / Difference / COGS account.
    pub expense_account: Option<String>,
}

/// Ledger Preview Engine simulating SLE & GL transactions.
pub struct LedgerPreviewEngine;

impl LedgerPreviewEngine {
    /// Generates preview ledger entries without modifying database state.
    pub fn simulate_stock_entry(
        _posting_date: NaiveDate,
        purpose: StockEntryPurpose,
        items: &[StockEntryLine],
    ) -> StockEntryLedgerPreview {
        let mut sles = Vec::new();
        let mut gls = Vec::new();

        for item in items {
            let line_value = item.qty * item.basic_rate;

            // Source Outward Movement
            if let Some(src_wh) = &item.source_warehouse {
                sles.push(SimulatedSle {
                    item_code: item.item_code.clone(),
                    warehouse: src_wh.clone(),
                    actual_qty: -item.qty,
                    valuation_rate: item.basic_rate,
                    stock_value_difference: -line_value,
                });

                if let Some(src_acc) = &item.source_stock_account {
                    gls.push(SimulatedGl {
                        account: src_acc.clone(),
                        debit: Decimal::ZERO,
                        credit: line_value,
                    });
                }
            }

            // Target Inward Movement
            if let Some(tgt_wh) = &item.target_warehouse {
                sles.push(SimulatedSle {
                    item_code: item.item_code.clone(),
                    warehouse: tgt_wh.clone(),
                    actual_qty: item.qty,
                    valuation_rate: item.basic_rate,
                    stock_value_difference: line_value,
                });

                if let Some(tgt_acc) = &item.target_stock_account {
                    gls.push(SimulatedGl {
                        account: tgt_acc.clone(),
                        debit: line_value,
                        credit: Decimal::ZERO,
                    });
                }
            }

            // Offset to Expense Account if Material Issue or Receipt
            if purpose == StockEntryPurpose::MaterialIssue {
                if let Some(exp_acc) = &item.expense_account {
                    gls.push(SimulatedGl {
                        account: exp_acc.clone(),
                        debit: line_value,
                        credit: Decimal::ZERO,
                    });
                }
            } else if purpose == StockEntryPurpose::MaterialReceipt {
                if let Some(exp_acc) = &item.expense_account {
                    gls.push(SimulatedGl {
                        account: exp_acc.clone(),
                        debit: Decimal::ZERO,
                        credit: line_value,
                    });
                }
            }
        }

        let total_debit: Decimal = gls.iter().map(|g| g.debit).sum();
        let total_credit: Decimal = gls.iter().map(|g| g.credit).sum();

        StockEntryLedgerPreview {
            stock_ledger_entries: sles,
            gl_entries: gls,
            total_debit,
            total_credit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_stock_entry_transfer_preview() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        let items = vec![StockEntryLine {
            item_code: "LAPTOP-PRO".into(),
            source_warehouse: Some("Main Stores".into()),
            target_warehouse: Some("Retail Shop".into()),
            qty: dec!(5),
            basic_rate: dec!(1000.0),
            source_stock_account: Some("1301 - Main Stores Stock".into()),
            target_stock_account: Some("1302 - Retail Shop Stock".into()),
            expense_account: None,
        }];

        let preview = LedgerPreviewEngine::simulate_stock_entry(
            date,
            StockEntryPurpose::MaterialTransfer,
            &items,
        );

        // SLE: -5 in Main Stores, +5 in Retail Shop
        assert_eq!(preview.stock_ledger_entries.len(), 2);
        assert_eq!(preview.stock_ledger_entries[0].actual_qty, dec!(-5));
        assert_eq!(preview.stock_ledger_entries[1].actual_qty, dec!(5));

        // GL: Debit Retail Shop 5000, Credit Main Stores 5000
        assert_eq!(preview.gl_entries.len(), 2);
        assert_eq!(preview.total_debit, dec!(5000.0));
        assert_eq!(preview.total_credit, dec!(5000.0));
        assert_eq!(preview.total_debit, preview.total_credit);
    }
}
