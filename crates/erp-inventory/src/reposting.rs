use crate::valuation::StockLedgerEntry;
use compact_str::CompactString;
use rayon::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Condvar, Mutex};

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

/// Unique identifier for an `(item_code, warehouse)` partition bin.
///
/// Implements canonical lexicographical ordering: first by `item_code`, then by `warehouse`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartitionKey {
    pub item_code: CompactString,
    pub warehouse: CompactString,
}

impl PartitionKey {
    #[must_use]
    pub fn new(item_code: impl Into<CompactString>, warehouse: impl Into<CompactString>) -> Self {
        Self {
            item_code: item_code.into(),
            warehouse: warehouse.into(),
        }
    }
}

/// Deadlock-free Ordered-Key Lock Coordinator for parallel stock ledger reposting.
///
/// Prevents cyclic lock inversion deadlocks during inter-warehouse transfers
/// by sorting partition keys alphabetically before acquiring locks.
#[derive(Default, Debug)]
pub struct OrderedPartitionLockManager {
    locked_keys: Mutex<BTreeSet<PartitionKey>>,
    condvar: Condvar,
}

/// RAII Guard holding exclusive write access across multiple partition bins in strict lexicographical order.
pub struct OrderedPartitionLocks<'a> {
    manager: &'a OrderedPartitionLockManager,
    pub locked_keys: Vec<PartitionKey>,
}

impl Drop for OrderedPartitionLocks<'_> {
    fn drop(&mut self) {
        let mut locked = self.manager.locked_keys.lock().unwrap();
        for key in &self.locked_keys {
            locked.remove(key);
        }
        self.manager.condvar.notify_all();
    }
}

impl OrderedPartitionLockManager {
    /// Creates a new lock manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sorts and deduplicates partition keys in canonical alphabetical order.
    #[must_use]
    pub fn canonicalize_keys(keys: &[PartitionKey]) -> Vec<PartitionKey> {
        let mut sorted = keys.to_vec();
        sorted.sort();
        sorted.dedup();
        sorted
    }

    /// Acquires write locks for an arbitrary collection of partition keys in strict alphabetical order.
    ///
    /// If any requested partition is currently locked, blocks until all requested partitions
    /// become available simultaneously, eliminating hold-and-wait deadlocks.
    pub fn acquire_locks<'a>(&'a self, keys: &[PartitionKey]) -> OrderedPartitionLocks<'a> {
        let sorted_keys = Self::canonicalize_keys(keys);
        if sorted_keys.is_empty() {
            return OrderedPartitionLocks {
                manager: self,
                locked_keys: Vec::new(),
            };
        }

        let mut locked = self.locked_keys.lock().unwrap();
        loop {
            let conflict = sorted_keys.iter().any(|k| locked.contains(k));
            if !conflict {
                for k in &sorted_keys {
                    locked.insert(k.clone());
                }
                break;
            }
            locked = self.condvar.wait(locked).unwrap();
        }

        OrderedPartitionLocks {
            manager: self,
            locked_keys: sorted_keys,
        }
    }

    /// Acquires write locks for an inter-warehouse transfer touching both source and destination warehouses.
    ///
    /// Guarantees that whether a transfer is WH-A -> WH-B or WH-B -> WH-A, locks are always
    /// acquired in sorted alphabetical order, preventing cyclic wait deadlocks.
    pub fn acquire_transfer_locks<'a>(
        &'a self,
        item_code: &str,
        source_warehouse: &str,
        target_warehouse: &str,
    ) -> OrderedPartitionLocks<'a> {
        let key1 = PartitionKey::new(item_code, source_warehouse);
        let key2 = PartitionKey::new(item_code, target_warehouse);
        self.acquire_locks(&[key1, key2])
    }

    /// Executes an operation within the critical section holding ordered partition locks.
    pub fn with_locks<R>(&self, keys: &[PartitionKey], op: impl FnOnce() -> R) -> R {
        let _guard = self.acquire_locks(keys);
        op()
    }

    /// Executes an inter-warehouse transfer operation holding ordered partition locks.
    pub fn with_transfer_locks<R>(
        &self,
        item_code: &str,
        source_warehouse: &str,
        target_warehouse: &str,
        op: impl FnOnce() -> R,
    ) -> R {
        let _guard = self.acquire_transfer_locks(item_code, source_warehouse, target_warehouse);
        op()
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

    /// Recalculates inter-warehouse stock ledger entries for a transfer voucher
    /// safely holding ordered partition locks across both warehouses to prevent deadlocks.
    pub fn repost_inter_warehouse_transfer(
        lock_mgr: &OrderedPartitionLockManager,
        item_code: &str,
        source_wh: &str,
        dest_wh: &str,
        source_entries: &mut [StockLedgerEntry],
        dest_entries: &mut [StockLedgerEntry],
    ) -> (usize, usize) {
        let _guard = lock_mgr.acquire_transfer_locks(item_code, source_wh, dest_wh);
        let src_count = Self::repost_single_partition(source_entries);
        let dst_count = Self::repost_single_partition(dest_entries);
        (src_count, dst_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use std::thread;

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

    #[test]
    fn test_ordered_partition_keys_canonical_sorting() {
        let k1 = PartitionKey::new("ITEM-Z", "WH-B");
        let k2 = PartitionKey::new("ITEM-A", "WH-Z");
        let k3 = PartitionKey::new("ITEM-A", "WH-A");
        let k4 = PartitionKey::new("ITEM-Z", "WH-A");

        let canonical = OrderedPartitionLockManager::canonicalize_keys(&[k1, k2, k3, k4]);
        assert_eq!(canonical.len(), 4);
        assert_eq!(canonical[0].item_code, "ITEM-A");
        assert_eq!(canonical[0].warehouse, "WH-A");
        assert_eq!(canonical[1].item_code, "ITEM-A");
        assert_eq!(canonical[1].warehouse, "WH-Z");
        assert_eq!(canonical[2].item_code, "ITEM-Z");
        assert_eq!(canonical[2].warehouse, "WH-A");
        assert_eq!(canonical[3].item_code, "ITEM-Z");
        assert_eq!(canonical[3].warehouse, "WH-B");
    }

    #[test]
    fn test_ordered_partition_lock_deadlock_freedom_under_high_concurrency() {
        let lock_manager = Arc::new(OrderedPartitionLockManager::new());
        let iterations = 100;
        let mut handles = Vec::new();

        // 10 threads running WH-A -> WH-B transfer
        for _ in 0..10 {
            let mgr = Arc::clone(&lock_manager);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    mgr.with_transfer_locks("WIDGET-01", "WH-A", "WH-B", || {
                        thread::yield_now();
                    });
                }
            }));
        }

        // 10 threads running inverse transfer WH-B -> WH-A simultaneously
        for _ in 0..10 {
            let mgr = Arc::clone(&lock_manager);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    mgr.with_transfer_locks("WIDGET-01", "WH-B", "WH-A", || {
                        thread::yield_now();
                    });
                }
            }));
        }

        for h in handles {
            h.join().expect("Thread joined without deadlocking");
        }
    }

    #[test]
    fn test_repost_inter_warehouse_transfer_execution() {
        let lock_manager = OrderedPartitionLockManager::new();
        let dt = NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        let mut src_entries = vec![
            StockLedgerEntry {
                name: "SLE-SRC-01".into(),
                item_code: "STEEL-BAR".into(),
                warehouse: "WH-RAW".into(),
                posting_datetime: dt,
                voucher_type: "Purchase Receipt".into(),
                voucher_no: "PR-01".into(),
                actual_qty: dec!(50),
                incoming_rate: dec!(20),
                outgoing_rate: dec!(0),
                stock_value_difference: dec!(0),
                qty_after_transaction: dec!(0),
                stock_value: dec!(0),
                valuation_rate: dec!(0),
            },
            StockLedgerEntry {
                name: "SLE-SRC-02".into(),
                item_code: "STEEL-BAR".into(),
                warehouse: "WH-RAW".into(),
                posting_datetime: dt,
                voucher_type: "Stock Entry".into(),
                voucher_no: "SE-XFER-01".into(),
                actual_qty: dec!(-20),
                incoming_rate: dec!(0),
                outgoing_rate: dec!(0),
                stock_value_difference: dec!(0),
                qty_after_transaction: dec!(0),
                stock_value: dec!(0),
                valuation_rate: dec!(0),
            },
        ];

        let mut dst_entries = vec![StockLedgerEntry {
            name: "SLE-DST-01".into(),
            item_code: "STEEL-BAR".into(),
            warehouse: "WH-FINISHED".into(),
            posting_datetime: dt,
            voucher_type: "Stock Entry".into(),
            voucher_no: "SE-XFER-01".into(),
            actual_qty: dec!(20),
            incoming_rate: dec!(20),
            outgoing_rate: dec!(0),
            stock_value_difference: dec!(0),
            qty_after_transaction: dec!(0),
            stock_value: dec!(0),
            valuation_rate: dec!(0),
        }];

        let (src_processed, dst_processed) =
            ParallelStockRepostingEngine::repost_inter_warehouse_transfer(
                &lock_manager,
                "STEEL-BAR",
                "WH-RAW",
                "WH-FINISHED",
                &mut src_entries,
                &mut dst_entries,
            );

        assert_eq!(src_processed, 2);
        assert_eq!(dst_processed, 1);
        assert_eq!(src_entries[1].qty_after_transaction, dec!(30));
        assert_eq!(src_entries[1].valuation_rate, dec!(20));
        assert_eq!(dst_entries[0].qty_after_transaction, dec!(20));
        assert_eq!(dst_entries[0].valuation_rate, dec!(20));
    }
}
