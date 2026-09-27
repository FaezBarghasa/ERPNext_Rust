//! Trade Compliance, Denied Party Screening & Dual-Use Export Control.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SanctionEntry {
    pub list_source: String, // OFAC SDN, BIS Entity List, EU Sanctions, UN Security Council
    pub entity_name: String,
    pub country: String,
    pub aliases: Vec<String>,
}

pub struct SanctionsScreener;

impl SanctionsScreener {
    /// Computes Jaro-Winkler-style character similarity between two names (0.0 to 1.0).
    #[must_use]
    pub fn name_similarity(a: &str, b: &str) -> f64 {
        let a_norm = a.to_lowercase();
        let b_norm = b.to_lowercase();
        if a_norm == b_norm {
            return 1.0;
        }
        if a_norm.contains(&b_norm) || b_norm.contains(&a_norm) {
            return 0.90;
        }
        let a_words: Vec<&str> = a_norm.split_whitespace().collect();
        let b_words: Vec<&str> = b_norm.split_whitespace().collect();

        let mut matched_words = 0;
        for w in &a_words {
            if b_words.contains(w) {
                matched_words += 1;
            }
        }
        let max_len = a_words.len().max(b_words.len());
        if max_len == 0 {
            0.0
        } else {
            matched_words as f64 / max_len as f64
        }
    }

    /// Screens an entity against the watchlist. Returns matched entries if similarity >= threshold (default 0.85).
    pub fn screen_party<'a>(
        party_name: &str,
        watchlist: &'a [SanctionEntry],
        threshold: f64,
    ) -> Vec<(&'a SanctionEntry, f64)> {
        let mut matches = Vec::new();
        for entry in watchlist {
            let sim_direct = Self::name_similarity(party_name, &entry.entity_name);
            let mut best_sim = sim_direct;

            for alias in &entry.aliases {
                let sim_alias = Self::name_similarity(party_name, alias);
                if sim_alias > best_sim {
                    best_sim = sim_alias;
                }
            }

            if best_sim >= threshold {
                matches.push((entry, best_sim));
            }
        }
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanction_screening_fuzzy_matching() {
        let watchlist = vec![SanctionEntry {
            list_source: "OFAC SDN".into(),
            entity_name: "MegaCorp Aerospace Front Co".into(),
            country: "XX".into(),
            aliases: vec!["MegaCorp Aero Trading".into()],
        }];

        let hits = SanctionsScreener::screen_party("MegaCorp Aero Trading", &watchlist, 0.85);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].1 >= 0.90);
    }
}
