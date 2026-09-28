use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
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
    pub fn new(partition_count: usize) -> Self {
        let count = partition_count.max(1);
        Self {
            partitions: HashMap::with_capacity(count),
            partition_count: count,
        }
    }

    /// Computes the target partition index for an item code.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
