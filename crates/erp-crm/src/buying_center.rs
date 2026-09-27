//! Account Relationship Graph, Buying Center Mapping & Corporate Hierarchy Rollups.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StakeholderRole {
    EconomicBuyer,
    TechnicalEvaluator,
    ExecutiveSponsor,
    Champion,
    Blocker,
    LegalGatekeeper,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stakeholder {
    pub id: String,
    pub name: String,
    pub title: String,
    pub account_id: String,
    pub role: StakeholderRole,
    pub influence_score: f64, // 0.0 to 1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfluenceEdge {
    pub from_contact_id: String,
    pub to_contact_id: String,
    pub weight: f64,
    pub stance: String, // "Champion", "Skeptical", "Neutral"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorporateHierarchy {
    pub entity_id: String,
    pub parent_entity_id: Option<String>,
    pub global_ultimate_owner_id: String,   // GUO
    pub domestic_ultimate_owner_id: String, // DUO
    pub credit_limit: Decimal,
    pub consolidated_pricing_tier: String,
}

pub struct BuyingCenterGraph {
    pub stakeholders: HashMap<String, Stakeholder>,
    pub influence_edges: Vec<InfluenceEdge>,
}

impl BuyingCenterGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stakeholders: HashMap::new(),
            influence_edges: Vec::new(),
        }
    }

    pub fn add_stakeholder(&mut self, s: Stakeholder) {
        self.stakeholders.insert(s.id.clone(), s);
    }

    pub fn add_edge(&mut self, edge: InfluenceEdge) {
        self.influence_edges.push(edge);
    }

    /// Evaluates account consensus score based on Champions vs Blockers weighted by influence.
    #[must_use]
    pub fn calculate_consensus_index(&self) -> f64 {
        let mut score = 0.0;
        for s in self.stakeholders.values() {
            let role_multiplier = match s.role {
                StakeholderRole::EconomicBuyer => 2.0,
                StakeholderRole::ExecutiveSponsor => 1.8,
                StakeholderRole::Champion => 1.5,
                StakeholderRole::TechnicalEvaluator => 1.0,
                StakeholderRole::LegalGatekeeper => 1.2,
                StakeholderRole::Blocker => -2.0,
            };
            score += s.influence_score * role_multiplier;
        }
        score
    }
}

impl Default for BuyingCenterGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buying_center_consensus() {
        let mut graph = BuyingCenterGraph::new();
        graph.add_stakeholder(Stakeholder {
            id: "c1".into(),
            name: "Alice VP".into(),
            title: "VP Engineering".into(),
            account_id: "acc1".into(),
            role: StakeholderRole::Champion,
            influence_score: 0.9,
        });
        graph.add_stakeholder(Stakeholder {
            id: "c2".into(),
            name: "Bob CFO".into(),
            title: "CFO".into(),
            account_id: "acc1".into(),
            role: StakeholderRole::EconomicBuyer,
            influence_score: 1.0,
        });

        let consensus = graph.calculate_consensus_index();
        assert!(consensus > 3.0);
    }
}
