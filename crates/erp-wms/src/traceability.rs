use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Movement record node in the serial / batch lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageNode {
    /// Serial number or Batch ID.
    pub identifier: String,
    /// Item code.
    pub item_code: String,
    /// Originating transaction type (e.g. "Purchase Receipt", "Work Order", "Delivery Note").
    pub voucher_type: String,
    /// Originating transaction number.
    pub voucher_no: String,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Quantity involved.
    pub qty: Decimal,
    /// Warehouse source.
    pub from_warehouse: Option<String>,
    /// Warehouse destination.
    pub to_warehouse: Option<String>,
}

/// Genealogy edge linking parent input materials to child output assemblies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenealogyLink {
    /// Parent raw material batch / serial identifier.
    pub parent_id: String,
    /// Parent item code.
    pub parent_item_code: String,
    /// Child manufactured assembly / finished good batch / serial identifier.
    pub child_id: String,
    /// Child item code.
    pub child_item_code: String,
    /// Manufacturing Work Order / Stock Entry voucher.
    pub manufacturing_voucher: String,
}

/// Comprehensive Serial No & Batch Traceability Engine.
#[derive(Debug, Default, Clone)]
pub struct TraceabilityEngine {
    /// Indexed movement nodes by identifier (batch / serial).
    pub nodes: HashMap<String, Vec<LineageNode>>,
    /// Parent -> Children links (for forward traceability).
    pub forward_links: HashMap<String, Vec<GenealogyLink>>,
    /// Child -> Parents links (for backward traceability).
    pub backward_links: HashMap<String, Vec<GenealogyLink>>,
}

impl TraceabilityEngine {
    /// Creates a new empty traceability engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a movement node.
    pub fn add_node(&mut self, node: LineageNode) {
        self.nodes.entry(node.identifier.clone()).or_default().push(node);
    }

    /// Registers a manufacturing genealogy link.
    pub fn add_genealogy_link(&mut self, link: GenealogyLink) {
        self.forward_links
            .entry(link.parent_id.clone())
            .or_default()
            .push(link.clone());

        self.backward_links
            .entry(link.child_id.clone())
            .or_default()
            .push(link);
    }

    /// Forward Traceability: Tracks a vendor raw material batch to all manufactured assemblies and customer deliveries.
    pub fn forward_trace(&self, root_batch_or_serial: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = vec![root_batch_or_serial.to_string()];

        while let Some(current) = queue.pop() {
            if visited.insert(current.clone()) {
                if let Some(links) = self.forward_links.get(&current) {
                    for link in links {
                        queue.push(link.child_id.clone());
                    }
                }
            }
        }

        visited.into_iter().collect()
    }

    /// Backward Traceability: Traces a defective customer unit backwards to its manufacturing work orders and supplier raw materials.
    pub fn backward_trace(&self, customer_batch_or_serial: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = vec![customer_batch_or_serial.to_string()];

        while let Some(current) = queue.pop() {
            if visited.insert(current.clone()) {
                if let Some(links) = self.backward_links.get(&current) {
                    for link in links {
                        queue.push(link.parent_id.clone());
                    }
                }
            }
        }

        visited.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_bidirectional_traceability() {
        let mut engine = TraceabilityEngine::new();
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();

        // 1. Raw material batch receipt from supplier
        engine.add_node(LineageNode {
            identifier: "BATCH-STEEL-001".into(),
            item_code: "RAW-STEEL".into(),
            voucher_type: "Purchase Receipt".into(),
            voucher_no: "PR-2026-01".into(),
            posting_date: date,
            qty: dec!(500),
            from_warehouse: None,
            to_warehouse: Some("Raw Stores".into()),
        });

        // 2. Sub-assembly: Raw steel converted to Shaft Batch
        engine.add_genealogy_link(GenealogyLink {
            parent_id: "BATCH-STEEL-001".into(),
            parent_item_code: "RAW-STEEL".into(),
            child_id: "BATCH-SHAFT-002".into(),
            child_item_code: "SUB-SHAFT".into(),
            manufacturing_voucher: "WO-2026-100".into(),
        });

        // 3. Final assembly: Shaft batch used in Motor Serial No
        engine.add_genealogy_link(GenealogyLink {
            parent_id: "BATCH-SHAFT-002".into(),
            parent_item_code: "SUB-SHAFT".into(),
            child_id: "SN-MOTOR-999".into(),
            child_item_code: "FG-MOTOR".into(),
            manufacturing_voucher: "WO-2026-200".into(),
        });

        // Forward Trace from Raw Steel
        let forward_impact = engine.forward_trace("BATCH-STEEL-001");
        assert!(forward_impact.contains(&"BATCH-STEEL-001".to_string()));
        assert!(forward_impact.contains(&"BATCH-SHAFT-002".to_string()));
        assert!(forward_impact.contains(&"SN-MOTOR-999".to_string()));

        // Backward Trace from Defective Motor Serial
        let backward_origin = engine.backward_trace("SN-MOTOR-999");
        assert!(backward_origin.contains(&"SN-MOTOR-999".to_string()));
        assert!(backward_origin.contains(&"BATCH-SHAFT-002".to_string()));
        assert!(backward_origin.contains(&"BATCH-STEEL-001".to_string()));
    }
}
