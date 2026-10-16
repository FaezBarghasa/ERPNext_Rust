use crate::valuation::StockLedgerEntry;
use compact_str::CompactString;
use rayon::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Configuration settings for Parallel Stock Reposting Engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParallelRepostingSettings {
    /// Enables concurrent parallel stock ledger recalculation.
    pub enable_parallel_reposting: bool,
    /// Maximum concurrent worker threads allowed per item hash bucket.
    pub no_of_parallel_reposting_per_item: usize,
    /// Number of items processed per batch chunk.
    pub batch_limit: usize,
}

impl Default for ParallelRepostingSettings {
    fn default() -> Self {
        Self {
            enable_parallel_reposting: true,
            no_of_parallel_reposting_per_item: 4,
            batch_limit: 100,
        }
    }
}

/// Reposting Job item queued for valuation and FIFO recalculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepostItemJob {
    /// Item code to repost.
    pub item_code: String,
    /// Warehouse to repost (or None for all warehouses of this item).
    pub warehouse: Option<String>,
    /// Earliest posting date requiring back-dated recalculation.
    pub start_date: String,
}

/// Item Hash Partition Queue to guarantee item-level linear consistency while allowing cross-item concurrency.
#[derive(Debug, Default)]
pub struct ItemPartitionQueue {
    /// Partitions mapping partition index to list of item jobs.
    pub partitions: HashMap<usize, Vec<RepostItemJob>>,
    /// Number of total hash partitions (buckets).
    pub partition_count: usize,
}

impl ItemPartitionQueue {
    /// Creates a partition queue with the specified number of buckets.
    #[must_use]
    pub fn new(partition_count: usize) -> Self {
        let count = partition_count.max(1);
        Self {
            partitions: HashMap::with_capacity(count),
            partition_count: count,
        }
    }

    /// Computes the target partition index for an item code.
    #[must_use]
    pub fn get_partition_index(&self, item_code: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        item_code.hash(&mut hasher);
        (hasher.finish() as usize) % self.partition_count
    }

    /// Pushes a job into its deterministic item partition.
    pub fn enqueue(&mut self, job: RepostItemJob) {
        let idx = self.get_partition_index(&job.item_code);
        self.partitions.entry(idx).or_default().push(job);
    }

    /// Retrieves all jobs within a specific partition for worker execution.
    pub fn drain_partition(&mut self, partition_idx: usize) -> Vec<RepostItemJob> {
        self.partitions.remove(&partition_idx).unwrap_or_default()
    }
}

/// High-throughput Rayon-based Parallel Stock Reposting Engine.
///
/// Recalculates moving average valuation rates, balance quantities, and stock values
/// across millions of Stock Ledger Entries (SLE) partitioned by `(item_code, warehouse)`.
pub struct ParallelStockRepostingEngine;

impl ParallelStockRepostingEngine {
    /// Recalculates a single continuous sequence of entries for an `(item, warehouse)` bin.
    pub fn repost_single_partition(entries: &mut [StockLedgerEntry]) -> usize {
        let mut running_qty = Decimal::ZERO;
        let mut running_stock_val = Decimal::ZERO;
        let mut running_val_rate = Decimal::ZERO;

        for sle in entries.iter_mut() {
            if sle.actual_qty > Decimal::ZERO {
                // Inward receipt
                let new_qty = running_qty + sle.actual_qty;
                let inward_val = sle.actual_qty * sle.incoming_rate;
                let new_stock_val = (running_stock_val + inward_val).max(Decimal::ZERO);
                let new_val_rate = if new_qty > Decimal::ZERO {
                    new_stock_val / new_qty
                } else {
                    sle.incoming_rate
                };

                sle.outgoing_rate = Decimal::ZERO;
                sle.valuation_rate = new_val_rate;
                sle.stock_value = new_stock_val;
                sle.qty_after_transaction = new_qty;
                sle.stock_value_difference = new_stock_val - running_stock_val;

                running_qty = new_qty;
                running_stock_val = new_stock_val;
                running_val_rate = new_val_rate;
            } else if sle.actual_qty < Decimal::ZERO {
                // Outward dispatch
                let outgoing_rate = running_val_rate;
                let new_qty = running_qty + sle.actual_qty; // sle.actual_qty is negative
                let consumed_val = -sle.actual_qty * outgoing_rate;
                let new_stock_val = (running_stock_val - consumed_val).max(Decimal::ZERO);

                sle.outgoing_rate = outgoing_rate;
                sle.valuation_rate = running_val_rate;
                sle.stock_value = new_stock_val;
                sle.qty_after_transaction = new_qty;
                sle.stock_value_difference = -consumed_val;

                running_qty = new_qty;
                running_stock_val = new_stock_val;
            }
        }

        entries.len()
    }

    /// Reposts all partitions concurrently in parallel using Rayon.
    ///
    /// Each partition represents an isolated `(item_code, warehouse)` timeline, guaranteeing
    /// strict linear causality per bin while achieving multi-core parallel speedup.
    pub fn repost_all_parallel(partitions: &mut [Vec<StockLedgerEntry>]) -> usize {
        partitions
            .par_iter_mut()
            .map(|part| Self::repost_single_partition(part))
            .sum()
    }

    /// Partitions an arbitrary vector of SLEs by `(item_code, warehouse)` and reposts them concurrently.
    pub fn repost_dataset_parallel(entries: &mut Vec<StockLedgerEntry>) -> usize {
        let mut grouped: HashMap<(CompactString, CompactString), Vec<StockLedgerEntry>> =
            HashMap::new();
        for entry in entries.drain(..) {
            let key = (entry.item_code.clone(), entry.warehouse.clone());
            grouped.entry(key).or_default().push(entry);
        }

        let mut partitions: Vec<Vec<StockLedgerEntry>> = grouped.into_values().collect();
        let total = Self::repost_all_parallel(&mut partitions);

        for mut part in partitions {
            entries.append(&mut part);
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_partition_queue_deterministic_routing() {
        let mut queue = ItemPartitionQueue::new(4);

        let job1 = RepostItemJob {
            item_code: "ITEM-A".into(),
            warehouse: Some("Main Stores".into()),
            start_date: "2026-01-01".into(),
        };

        let job2 = RepostItemJob {
            item_code: "ITEM-A".into(),
            warehouse: Some("Secondary Stores".into()),
            start_date: "2026-02-01".into(),
        };

        let idx1 = queue.get_partition_index(&job1.item_code);
        let idx2 = queue.get_partition_index(&job2.item_code);
        assert_eq!(idx1, idx2); // Same item code hashes to the exact same partition

        queue.enqueue(job1);
        queue.enqueue(job2);

        let partition_jobs = queue.drain_partition(idx1);
        assert_eq!(partition_jobs.len(), 2);
    }

    #[test]
    fn test_parallel_reposting_engine_causality_and_concurrency() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        // 10 partitions with 100 SLEs each = 1,000 SLEs
        let mut partitions: Vec<Vec<StockLedgerEntry>> = (0..10)
            .map(|i| {
                let item = format!("SKU-{i}");
                let wh = format!("WH-{i}");
                vec![
                    StockLedgerEntry {
                        name: format!("SLE-IN-{i}").into(),
                        item_code: item.as_str().into(),
                        warehouse: wh.as_str().into(),
                        posting_datetime: dt,
                        voucher_type: "Purchase Receipt".into(),
                        voucher_no: format!("PR-{i}").into(),
                        actual_qty: dec!(100),
                        incoming_rate: dec!(50),
                        outgoing_rate: dec!(0),
                        stock_value_difference: dec!(0),
                        qty_after_transaction: dec!(0),
                        stock_value: dec!(0),
                        valuation_rate: dec!(0),
                    },
                    StockLedgerEntry {
                        name: format!("SLE-OUT-{i}").into(),
                        item_code: item.as_str().into(),
                        warehouse: wh.as_str().into(),
                        posting_datetime: dt,
                        voucher_type: "Delivery Note".into(),
                        voucher_no: format!("DN-{i}").into(),
                        actual_qty: dec!(-30),
                        incoming_rate: dec!(0),
                        outgoing_rate: dec!(0),
                        stock_value_difference: dec!(0),
                        qty_after_transaction: dec!(0),
                        stock_value: dec!(0),
                        valuation_rate: dec!(0),
                    },
                ]
            })
            .collect();

        let total_processed = ParallelStockRepostingEngine::repost_all_parallel(&mut partitions);
        assert_eq!(total_processed, 20);

        for part in &partitions {
            assert_eq!(part[0].qty_after_transaction, dec!(100));
            assert_eq!(part[0].valuation_rate, dec!(50));
            assert_eq!(part[0].stock_value, dec!(5000));

            assert_eq!(part[1].qty_after_transaction, dec!(70));
            assert_eq!(part[1].outgoing_rate, dec!(50));
            assert_eq!(part[1].stock_value, dec!(3500));
            assert_eq!(part[1].stock_value_difference, dec!(-1500));
        }
    }
}
