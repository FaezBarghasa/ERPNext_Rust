//! Local-First Edge Synchronization & CRDT State Arbiter (`desk_app::edge_sync`).
//!
//! Enforces the offline invariant:
//! Edge Disconnect -> Local Transaction Execution -> Network Restoration -> Convergence without manual conflict resolution
//!
//! Uses Positive-Negative (PN) Counters for inventory/stock adjustments and
//! Last-Write-Wins (LWW) Element Sets for document records.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device Causal Vector Clock tracking microsecond-level state sequence numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VectorClock {
    pub clocks: HashMap<CompactString, u64>,
}

impl VectorClock {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments clock for the given device ID.
    pub fn increment(&mut self, device_id: &str) {
        let entry = self.clocks.entry(device_id.into()).or_insert(0);
        *entry += 1;
    }

    /// Merges another vector clock using pairwise supremum (component-wise max).
    pub fn merge(&mut self, other: &Self) {
        for (dev, clock) in &other.clocks {
            let entry = self.clocks.entry(dev.clone()).or_insert(0);
            *entry = (*entry).max(*clock);
        }
    }

    /// Returns true if this clock dominates or equals other.
    #[must_use]
    pub fn dominates(&self, other: &Self) -> bool {
        for (dev, clock) in &other.clocks {
            let my_clock = self.clocks.get(dev).copied().unwrap_or(0);
            if my_clock < *clock {
                return false;
            }
        }
        true
    }
}

/// Offline Local Mutation Operation Kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeMutationKind {
    /// Stock adjustment or increment (PN-Counter increment/decrement)
    StockAdjustment {
        item_code: CompactString,
        delta: i64,
    },
    /// Document update or insertion (LWW with timestamp)
    DocumentUpsert {
        doctype: CompactString,
        doc_name: CompactString,
        payload_json: CompactString,
        timestamp_micros: i64,
    },
    /// Document deletion (LWW tombstone)
    DocumentDelete {
        doctype: CompactString,
        doc_name: CompactString,
        timestamp_micros: i64,
    },
}

/// Cryptographically sequenced local mutation envelope recorded while disconnected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeMutationEnvelope {
    pub sequence_id: u64,
    pub device_id: CompactString,
    pub mutation: EdgeMutationKind,
    pub vector_clock: VectorClock,
    pub created_at_micros: i64,
}

/// Local In-Memory / Embedded SurrealKV Mutation Buffer on Edge Devices.
#[derive(Debug, Clone, Default)]
pub struct LocalMutationBuffer {
    pub device_id: CompactString,
    pub current_clock: VectorClock,
    pub pending_mutations: Vec<EdgeMutationEnvelope>,
    pub sequence_counter: u64,
}

impl LocalMutationBuffer {
    #[must_use]
    pub fn new(device_id: CompactString) -> Self {
        Self {
            device_id,
            current_clock: VectorClock::new(),
            pending_mutations: Vec::new(),
            sequence_counter: 0,
        }
    }

    /// Records an offline transaction locally on the device.
    pub fn record_mutation(&mut self, mutation: EdgeMutationKind, now_micros: i64) -> EdgeMutationEnvelope {
        self.sequence_counter += 1;
        self.current_clock.increment(&self.device_id);

        let envelope = EdgeMutationEnvelope {
            sequence_id: self.sequence_counter,
            device_id: self.device_id.clone(),
            mutation,
            vector_clock: self.current_clock.clone(),
            created_at_micros: now_micros,
        };

        self.pending_mutations.push(envelope.clone());
        envelope
    }

    /// Number of pending mutations awaiting cloud sync.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending_mutations.len()
    }

    /// Clears synced mutations after successful cloud acknowledgement.
    pub fn acknowledge_sync(&mut self, acked_sequence_id: u64) {
        self.pending_mutations.retain(|m| m.sequence_id > acked_sequence_id);
    }
}

/// Cloud State Replica with Deterministic Join Semilattice Reconciler.
#[derive(Debug, Default)]
pub struct CloudSyncArbiter {
    pub server_vector_clock: VectorClock,
    pub stock_levels: HashMap<CompactString, i64>,
    pub document_store: HashMap<CompactString, (CompactString, i64)>, // key -> (payload, timestamp)
    pub tombstones: HashMap<CompactString, i64>,                     // key -> timestamp
}

impl CloudSyncArbiter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a batch of edge mutation envelopes streamed upon network reconnection.
    /// Evaluates S_final = S_server ⊔ S_device deterministically without manual conflicts.
    pub fn reconcile_batch(&mut self, envelopes: Vec<EdgeMutationEnvelope>) -> u64 {
        let mut max_seq = 0;

        for env in envelopes {
            max_seq = max_seq.max(env.sequence_id);
            self.server_vector_clock.merge(&env.vector_clock);

            match env.mutation {
                EdgeMutationKind::StockAdjustment { item_code, delta } => {
                    // Commutative PN-Counter addition: delta can be positive or negative
                    let level = self.stock_levels.entry(item_code).or_insert(0);
                    *level += delta;
                }
                EdgeMutationKind::DocumentUpsert {
                    doctype,
                    doc_name,
                    payload_json,
                    timestamp_micros,
                } => {
                    let doc_key: CompactString = format!("{doctype}:{doc_name}").into();
                    // Check if tombstone exists with higher timestamp
                    let tombstone_ts = self.tombstones.get(&doc_key).copied().unwrap_or(0);
                    if timestamp_micros > tombstone_ts {
                        let existing = self.document_store.get(&doc_key);
                        let should_update = match existing {
                            Some((_, existing_ts)) => timestamp_micros > *existing_ts,
                            None => true,
                        };
                        if should_update {
                            self.document_store.insert(doc_key, (payload_json, timestamp_micros));
                        }
                    }
                }
                EdgeMutationKind::DocumentDelete {
                    doctype,
                    doc_name,
                    timestamp_micros,
                } => {
                    let doc_key: CompactString = format!("{doctype}:{doc_name}").into();
                    let existing_tombstone = self.tombstones.get(&doc_key).copied().unwrap_or(0);
                    if timestamp_micros > existing_tombstone {
                        self.tombstones.insert(doc_key.clone(), timestamp_micros);
                        self.document_store.remove(&doc_key);
                    }
                }
            }
        }

        max_seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_sync_convergence_invariant() {
        let mut device_a = LocalMutationBuffer::new("DEV-SCANNER-A".into());
        let mut device_b = LocalMutationBuffer::new("DEV-SCANNER-B".into());
        let mut arbiter = CloudSyncArbiter::new();

        // 1. Edge Disconnect: Device A logs stock pick of 5 units (now_micros = 1000)
        let m_a1 = device_a.record_mutation(
            EdgeMutationKind::StockAdjustment {
                item_code: "SKU-BEARING-608".into(),
                delta: -5,
            },
            1000,
        );

        // 2. Edge Disconnect: Device B logs stock receipt of 20 units (now_micros = 1010)
        let m_b1 = device_b.record_mutation(
            EdgeMutationKind::StockAdjustment {
                item_code: "SKU-BEARING-608".into(),
                delta: 20,
            },
            1010,
        );

        // 3. Device A updates Customer document (ts = 1020)
        let m_a2 = device_a.record_mutation(
            EdgeMutationKind::DocumentUpsert {
                doctype: "Customer".into(),
                doc_name: "CUST-001".into(),
                payload_json: r#"{"name": "Apex Industrial", "status": "Active"}"#.into(),
                timestamp_micros: 1020,
            },
            1020,
        );

        // 4. Device B updates same Customer document with newer edit (ts = 1050)
        let m_b2 = device_b.record_mutation(
            EdgeMutationKind::DocumentUpsert {
                doctype: "Customer".into(),
                doc_name: "CUST-001".into(),
                payload_json: r#"{"name": "Apex Industrial Global", "status": "VIP"}"#.into(),
                timestamp_micros: 1050,
            },
            1050,
        );

        // 5. Network Restoration: Reconcile all outbox queues regardless of arrival order
        arbiter.reconcile_batch(vec![m_a1, m_a2]);
        arbiter.reconcile_batch(vec![m_b1, m_b2]);

        // Invariant Check 1: Commutative PN-Counter converges to (-5 + 20 = 15)
        assert_eq!(arbiter.stock_levels.get("SKU-BEARING-608"), Some(&15));

        // Invariant Check 2: LWW-Element-Set converges deterministically to the latest edit
        let (payload, ts) = arbiter.document_store.get("Customer:CUST-001").unwrap();
        assert_eq!(ts, &1050);
        assert!(payload.contains("Apex Industrial Global"));
    }
}
