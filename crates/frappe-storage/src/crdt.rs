//! Conflict-Free Replicated Data Types (CRDTs) & Offline Edge Replication (`frappe-storage::crdt`).
//!
//! Implements state-based CRDTs (PN-Counters, LWW-Element-Sets, Vector Clocks)
//! and offline outbox buffers for edge devices (POS tablets, kiosks) synchronizing
//! with central SurrealDB clusters upon reconnection.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Vector Clock for causality tracking across edge nodes ($\vec{V}_k$).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock {
    pub clocks: BTreeMap<CompactString, u64>,
}

impl VectorClock {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments node logical timestamp.
    pub fn increment(&mut self, node_id: &str) {
        let counter = self.clocks.entry(node_id.into()).or_insert(0);
        *counter += 1;
    }

    /// Joins two vector clocks taking the maximum value for each node.
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, counter) in &other.clocks {
            let entry = self.clocks.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(*counter);
        }
    }

    /// Returns true if this clock is causally ahead of or concurrent with other.
    #[must_use]
    pub fn is_concurrent_or_ahead(&self, other: &VectorClock) -> bool {
        for (node, other_counter) in &other.clocks {
            if let Some(self_counter) = self.clocks.get(node) {
                if self_counter < other_counter {
                    return false;
                }
            } else if *other_counter > 0 {
                return false;
            }
        }
        true
    }
}

/// Positive-Negative Counter CRDT (PN-Counter).
/// Allows independent increment and decrement operations across nodes that commute.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PnCounter {
    pub positive: BTreeMap<CompactString, u64>,
    pub negative: BTreeMap<CompactString, u64>,
}

impl PnCounter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments counter by amount for node.
    pub fn inc(&mut self, node_id: &str, amount: u64) {
        let p = self.positive.entry(node_id.into()).or_insert(0);
        *p += amount;
    }

    /// Decrements counter by amount for node.
    pub fn dec(&mut self, node_id: &str, amount: u64) {
        let n = self.negative.entry(node_id.into()).or_insert(0);
        *n += amount;
    }

    /// Evaluates current global counter value: $\sum P - \sum N$.
    #[must_use]
    pub fn value(&self) -> i64 {
        let p_sum: u64 = self.positive.values().sum();
        let n_sum: u64 = self.negative.values().sum();
        p_sum as i64 - n_sum as i64
    }

    /// Merges two PN-Counters using the join-semilattice ($S_1 \sqcup S_2$).
    pub fn merge(&mut self, other: &PnCounter) {
        for (node, count) in &other.positive {
            let p = self.positive.entry(node.clone()).or_insert(0);
            *p = (*p).max(*count);
        }
        for (node, count) in &other.negative {
            let n = self.negative.entry(node.clone()).or_insert(0);
            *n = (*n).max(*count);
        }
    }
}

/// Last-Write-Wins Element Set (LWW-Element-Set) for master document updates.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LwwElement<T> {
    pub value: T,
    pub timestamp_micros: i64,
    pub node_id: CompactString,
}

/// Last-Write-Wins Register Map for document fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LwwDocumentState {
    pub fields: HashMap<CompactString, LwwElement<serde_json::Value>>,
}

impl LwwDocumentState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets field with timestamp and node origin.
    pub fn set_field(
        &mut self,
        field_name: impl Into<CompactString>,
        value: serde_json::Value,
        node_id: impl Into<CompactString>,
        timestamp: DateTime<Utc>,
    ) {
        let key = field_name.into();
        let micros = timestamp.timestamp_micros();
        let node = node_id.into();

        if let Some(existing) = self.fields.get(&key) {
            if existing.timestamp_micros > micros {
                return; // Existing write is newer
            }
            if existing.timestamp_micros == micros && existing.node_id > node {
                return; // Deterministic tie-breaking on node_id
            }
        }

        self.fields.insert(
            key,
            LwwElement {
                value,
                timestamp_micros: micros,
                node_id: node,
            },
        );
    }

    /// Merges remote document state.
    pub fn merge(&mut self, other: &LwwDocumentState) {
        for (field, other_elem) in &other.fields {
            if let Some(self_elem) = self.fields.get(field) {
                if other_elem.timestamp_micros > self_elem.timestamp_micros
                    || (other_elem.timestamp_micros == self_elem.timestamp_micros
                        && other_elem.node_id > self_elem.node_id)
                {
                    self.fields.insert(field.clone(), other_elem.clone());
                }
            } else {
                self.fields.insert(field.clone(), other_elem.clone());
            }
        }
    }
}

/// Offline Replication Outbox Queue Entry (`sys_sync_queue`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SyncQueueEntry {
    pub sync_id: CompactString,
    pub tenant_id: CompactString,
    pub doctype: CompactString,
    pub doc_name: CompactString,
    pub action: CompactString, // "INSERT", "UPDATE", "SUBMIT"
    pub payload_json: CompactString,
    pub vector_clock: VectorClock,
    pub created_at: DateTime<Utc>,
    pub is_synced: bool,
}

/// Offline Edge Outbox Queue Manager.
#[derive(Clone, Debug, Default)]
pub struct OfflineOutboxManager {
    queue: Vec<SyncQueueEntry>,
}

impl OfflineOutboxManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new mutation to the offline replication queue.
    #[allow(clippy::too_many_arguments)]
    pub fn enqueue_mutation(
        &mut self,
        tenant_id: impl Into<CompactString>,
        doctype: impl Into<CompactString>,
        doc_name: impl Into<CompactString>,
        action: impl Into<CompactString>,
        payload_json: impl Into<CompactString>,
        mut clock: VectorClock,
        node_id: &str,
    ) -> SyncQueueEntry {
        clock.increment(node_id);
        let doc_name_str: CompactString = doc_name.into();
        let entry = SyncQueueEntry {
            sync_id: format!(
                "sync_{}_{}",
                doc_name_str.as_str(),
                Utc::now().timestamp_millis()
            )
            .into(),
            tenant_id: tenant_id.into(),
            doctype: doctype.into(),
            doc_name: doc_name_str,
            action: action.into(),
            payload_json: payload_json.into(),
            vector_clock: clock,
            created_at: Utc::now(),
            is_synced: false,
        };
        self.queue.push(entry.clone());
        entry
    }

    /// Drains unsynced transactions for cloud WebSocket synchronization.
    pub fn get_pending_sync(&self) -> Vec<SyncQueueEntry> {
        self.queue
            .iter()
            .filter(|e| !e.is_synced)
            .cloned()
            .collect()
    }

    /// Marks entries as acknowledged by the central SurrealDB cluster.
    pub fn mark_synced(&mut self, sync_ids: &[&str]) {
        for entry in &mut self.queue {
            if sync_ids.contains(&entry.sync_id.as_str()) {
                entry.is_synced = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_merging() {
        let mut v1 = VectorClock::new();
        v1.increment("pos_node_1");
        v1.increment("pos_node_1");

        let mut v2 = VectorClock::new();
        v2.increment("pos_node_2");

        v1.merge(&v2);
        assert_eq!(*v1.clocks.get("pos_node_1").unwrap(), 2);
        assert_eq!(*v1.clocks.get("pos_node_2").unwrap(), 1);
    }

    #[test]
    fn test_pn_counter_convergence() {
        let mut c1 = PnCounter::new();
        c1.inc("pos_1", 10);
        c1.dec("pos_1", 2); // 8

        let mut c2 = PnCounter::new();
        c2.inc("pos_2", 5);
        c2.dec("pos_2", 1); // 4

        c1.merge(&c2);
        c2.merge(&c1);

        assert_eq!(c1.value(), 12);
        assert_eq!(c2.value(), 12);
    }

    #[test]
    fn test_lww_document_convergence() {
        let mut s1 = LwwDocumentState::new();
        let now = Utc::now();
        s1.set_field(
            "customer_name",
            serde_json::json!("Alice Corp"),
            "pos_1",
            now,
        );

        let mut s2 = LwwDocumentState::new();
        let later = now + chrono::Duration::milliseconds(50);
        s2.set_field(
            "customer_name",
            serde_json::json!("Alice Global"),
            "pos_2",
            later,
        );

        s1.merge(&s2);
        assert_eq!(
            s1.fields.get("customer_name").unwrap().value,
            serde_json::json!("Alice Global")
        );
    }

    #[test]
    fn test_offline_outbox_queue() {
        let mut outbox = OfflineOutboxManager::new();
        let clock = VectorClock::new();

        let entry = outbox.enqueue_mutation(
            "tenant_retail",
            "SalesInvoice",
            "INV-POS-001",
            "SUBMIT",
            r#"{"grand_total": 99.50}"#,
            clock,
            "tablet_1",
        );

        let pending = outbox.get_pending_sync();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].sync_id, entry.sync_id);

        outbox.mark_synced(&[entry.sync_id.as_str()]);
        assert_eq!(outbox.get_pending_sync().len(), 0);
    }
}
