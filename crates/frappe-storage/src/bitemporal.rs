//! Bi-temporal state ledger supporting system time (transaction time) and valid time (business time).

use crate::merkle::MerkleHasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An open or bounded interval of time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeInterval {
    pub from: DateTime<Utc>,
    pub to: Option<DateTime<Utc>>,
}

impl TimeInterval {
    #[must_use]
    pub fn new(from: DateTime<Utc>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }

    #[must_use]
    pub fn point(at: DateTime<Utc>) -> Self {
        Self {
            from: at,
            to: Some(at),
        }
    }

    #[must_use]
    pub fn is_active_at(&self, point: DateTime<Utc>) -> bool {
        if point < self.from {
            return false;
        }
        match self.to {
            Some(t) => point <= t,
            None => true,
        }
    }
}

/// A bi-temporal EAV record storing both system commit time and business fact valid time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiTemporalRecord {
    pub id: String,
    pub doc_id: String,
    pub doctype: String,
    pub system_time: TimeInterval,
    pub valid_time: TimeInterval,
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub prev_hash: String,
    pub merkle_hash: String,
}

impl BiTemporalRecord {
    pub fn new(
        id: String,
        doc_id: String,
        doctype: String,
        system_time: TimeInterval,
        valid_time: TimeInterval,
        attributes: BTreeMap<String, serde_json::Value>,
        prev_hash: String,
    ) -> Self {
        let mut rec = Self {
            id,
            doc_id,
            doctype,
            system_time,
            valid_time,
            attributes,
            prev_hash,
            merkle_hash: String::new(),
        };
        rec.merkle_hash = rec.compute_hash();
        rec
    }

    #[must_use]
    pub fn compute_hash(&self) -> String {
        let mut hasher = MerkleHasher::new();
        hasher.update(self.doc_id.as_bytes());
        hasher.update(self.doctype.as_bytes());
        hasher.update(self.system_time.from.to_rfc3339().as_bytes());
        hasher.update(self.valid_time.from.to_rfc3339().as_bytes());
        let attrs = serde_json::to_string(&self.attributes).unwrap_or_default();
        hasher.update(attrs.as_bytes());
        hasher.update(self.prev_hash.as_bytes());
        hasher.finalize_hex()
    }

    /// Verifies tamper resistance against previous block in the chain.
    #[must_use]
    pub fn verify_integrity(&self, expected_prev_hash: &str) -> bool {
        self.prev_hash == expected_prev_hash && self.merkle_hash == self.compute_hash()
    }
}

/// Bi-Temporal query builder for SurrealQL.
pub struct BiTemporalQuery;

impl BiTemporalQuery {
    /// Builds a SurrealQL statement fetching records as of a specific system time and valid time.
    #[must_use]
    pub fn as_of(
        doctype: &str,
        doc_id: &str,
        system_as_of: DateTime<Utc>,
        valid_as_of: DateTime<Utc>,
    ) -> String {
        let sys_str = system_as_of.to_rfc3339();
        let val_str = valid_as_of.to_rfc3339();
        format!(
            "SELECT * FROM sys_bitemporal WHERE doctype = '{doctype}' AND doc_id = '{doc_id}' \
             AND system_time.from <= d'{sys_str}' AND (system_time.to IS NONE OR system_time.to >= d'{sys_str}') \
             AND valid_time.from <= d'{val_str}' AND (valid_time.to IS NONE OR valid_time.to >= d'{val_str}') \
             ORDER BY system_time.from DESC LIMIT 1;"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitemporal_hashing_and_intervals() {
        let t0 = Utc::now();
        let mut attrs = BTreeMap::new();
        attrs.insert("status".into(), serde_json::json!("Draft"));
        attrs.insert("grand_total".into(), serde_json::json!(15000.0));

        let rec = BiTemporalRecord::new(
            "rec_1".into(),
            "INV-2026-001".into(),
            "Sales Invoice".into(),
            TimeInterval::new(t0, None),
            TimeInterval::new(t0, None),
            attrs,
            "0000000000000000".into(),
        );

        assert!(!rec.merkle_hash.is_empty());
        assert!(rec.verify_integrity("0000000000000000"));
        assert!(rec.system_time.is_active_at(t0));
    }
}
