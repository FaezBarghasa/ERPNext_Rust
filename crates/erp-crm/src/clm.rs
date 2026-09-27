//! Contract Lifecycle Management (CLM), Semantic Clause Library & Redline Diffing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClauseVariant {
    Standard,
    PreferredFallback,
    ExtremeException, // Auto-escalates to General Counsel
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractClause {
    pub clause_id: String,
    pub title: String,
    pub jurisdiction: String,
    pub variant: ClauseVariant,
    pub template_text: String,
}

pub struct ContractRedliner;

impl ContractRedliner {
    /// Compares standard clause tokens against inbound third-party legal redlines.
    pub fn diff_tokens(standard: &str, inbound: &str) -> (Vec<String>, Vec<String>) {
        let std_words: Vec<&str> = standard.split_whitespace().collect();
        let in_words: Vec<&str> = inbound.split_whitespace().collect();

        let std_set: std::collections::HashSet<&str> = std_words.iter().copied().collect();
        let in_set: std::collections::HashSet<&str> = in_words.iter().copied().collect();

        let added: Vec<String> = in_set.difference(&std_set).map(ToString::to_string).collect();
        let removed: Vec<String> = std_set.difference(&in_set).map(ToString::to_string).collect();

        (added, removed)
    }
}

pub struct ClauseLibrary {
    clauses: HashMap<String, Vec<ContractClause>>,
}

impl ClauseLibrary {
    #[must_use]
    pub fn new() -> Self {
        Self {
            clauses: HashMap::new(),
        }
    }

    pub fn register(&mut self, clause: ContractClause) {
        self.clauses
            .entry(clause.title.clone())
            .or_default()
            .push(clause);
    }

    #[must_use]
    pub fn get_clause(&self, title: &str, variant: ClauseVariant) -> Option<&ContractClause> {
        self.clauses
            .get(title)?
            .iter()
            .find(|c| c.variant == variant)
    }
}

impl Default for ClauseLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_registration_and_diffing() {
        let mut lib = ClauseLibrary::new();
        lib.register(ContractClause {
            clause_id: "INDEM-STD".into(),
            title: "Indemnification".into(),
            jurisdiction: "US-DE".into(),
            variant: ClauseVariant::Standard,
            template_text: "Vendor shall indemnify Customer against third-party IP claims.".into(),
        });

        let clause = lib.get_clause("Indemnification", ClauseVariant::Standard).unwrap();
        assert_eq!(clause.clause_id, "INDEM-STD");

        let (added, removed) = ContractRedliner::diff_tokens(
            "Vendor shall indemnify Customer",
            "Vendor shall unconditionally indemnify and defend Customer",
        );
        assert!(added.contains(&"unconditionally".to_string()));
        assert!(added.contains(&"defend".to_string()));
        assert!(removed.is_empty());
    }
}
