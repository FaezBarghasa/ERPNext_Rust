//! Multi-Tier Intercompany Balancing & Automated Consolidation Elimination.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntercompanyTransaction {
    pub transaction_id: String,
    pub seller_entity_id: String,
    pub buyer_entity_id: String,
    pub amount: Decimal,
    pub cost_of_goods_sold: Decimal,
    pub currency: String,
    pub is_inventory_movement: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EliminationJournalEntry {
    pub debit_account: String,
    pub credit_account: String,
    pub elimination_amount: Decimal,
    pub explanation: String,
}

pub struct IntercompanyEliminationEngine;

impl IntercompanyEliminationEngine {
    /// Generates GAAP/IFRS consolidation eliminations for intercompany sales and inventory profit.
    pub fn generate_eliminations(
        txs: &[IntercompanyTransaction],
        remaining_inventory_ratio: Decimal, // % of intercompany goods still in buyer warehouse
    ) -> Vec<EliminationJournalEntry> {
        let mut eliminations = Vec::new();

        for tx in txs {
            // 1. Eliminate Intercompany Sales Revenue vs IC COGS
            eliminations.push(EliminationJournalEntry {
                debit_account: "IC Sales Revenue (Seller)".into(),
                credit_account: "IC COGS / Expense (Buyer)".into(),
                elimination_amount: tx.amount,
                explanation: format!(
                    "Eliminate IC revenue between {} and {}",
                    tx.seller_entity_id, tx.buyer_entity_id
                ),
            });

            // 2. Eliminate Unrealized Intercompany Inventory Profit if inventory is still held
            if tx.is_inventory_movement && !remaining_inventory_ratio.is_zero() {
                let gross_profit = tx.amount - tx.cost_of_goods_sold;
                if gross_profit > Decimal::ZERO {
                    let unrealized_profit = (gross_profit * remaining_inventory_ratio).round_dp(2);
                    eliminations.push(EliminationJournalEntry {
                        debit_account: "Consolidated Retained Earnings / Profit".into(),
                        credit_account: "Inventory Asset Valuation (Markup)".into(),
                        elimination_amount: unrealized_profit,
                        explanation: "Elimination of unrealized intercompany markup profit".into(),
                    });
                }
            }
        }

        eliminations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_intercompany_elimination_generation() {
        let txs = vec![IntercompanyTransaction {
            transaction_id: "IC-001".into(),
            seller_entity_id: "Sub_Germany".into(),
            buyer_entity_id: "Sub_USA".into(),
            amount: dec!(100000),
            cost_of_goods_sold: dec!(70000), // $30k markup
            currency: "USD".into(),
            is_inventory_movement: true,
        }];

        // 50% of purchased goods remain in Sub_USA stock at year-end
        let elims = IntercompanyEliminationEngine::generate_eliminations(&txs, dec!(0.50));
        assert_eq!(elims.len(), 2);
        assert_eq!(elims[0].elimination_amount, dec!(100000));
        assert_eq!(elims[1].elimination_amount, dec!(15000.00)); // 50% of $30k
    }
}
