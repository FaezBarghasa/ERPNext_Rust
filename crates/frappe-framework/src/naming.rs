//! Naming Series Engine (`frappe-framework::naming`).
//!
//! Provides thread-safe, concurrency-locked sequential document identifier generation,
//! series template formatting (.YYYY., .MM., .#####), field-interpolated autoname,
//! and collision-free counter allocation.

use crate::lifecycle::{Document, DocumentError};
use chrono::{Datelike, Utc};
use frappe_meta::DocTypeSchema;
use std::collections::HashMap;
use std::sync::Mutex;

/// Concurrent in-memory series counter store with thread safety.
#[derive(Default)]
pub struct NamingSeriesStore {
    counters: Mutex<HashMap<String, u64>>,
}

impl NamingSeriesStore {
    /// Creates a new in-memory series counter store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically increments and returns the next sequential number for a given prefix.
    pub fn get_next(&self, prefix: &str) -> u64 {
        let mut map = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        let entry = map.entry(prefix.to_string()).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Explicitly sets the counter for a given series prefix.
    pub fn set_counter(&self, prefix: &str, value: u64) {
        let mut map = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(prefix.to_string(), value);
    }

    /// Reads current counter without mutating.
    pub fn get_current(&self, prefix: &str) -> u64 {
        let map = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        map.get(prefix).copied().unwrap_or(0)
    }
}

/// Naming Engine resolving document keys per DocType schema and naming rules.
pub struct NamingEngine;

impl NamingEngine {
    /// Resolves and assigns a canonical `name` identifier to a new document.
    pub fn autoname(
        doc: &mut Document,
        schema: &DocTypeSchema,
        store: &NamingSeriesStore,
    ) -> Result<String, DocumentError> {
        // If user already supplied a valid name and manual naming or allow_rename is permitted
        if !doc.name.is_empty() {
            return Ok(doc.name.clone());
        }

        let now = Utc::now();
        let year = now.year();
        let month = now.month();
        let day = now.day();

        // 1. Check schema naming rule
        if let Some(rule) = &schema.naming_rule {
            if rule.starts_with("field:") {
                let fieldname = rule.trim_start_matches("field:").trim();
                if let Some(val) = doc.data.get(fieldname).and_then(|v| v.as_str()) {
                    let name = val.trim().to_string();
                    doc.name = name.clone();
                    return Ok(name);
                }
            } else if rule.contains(".#") {
                let name = Self::parse_series_template(rule, year, month, day, store);
                doc.name = name.clone();
                return Ok(name);
            }
        }

        // 2. Check document's own `naming_series` field if present
        if let Some(series_val) = doc.data.get("naming_series").and_then(|v| v.as_str()) {
            if series_val.contains(".#") {
                let name = Self::parse_series_template(series_val, year, month, day, store);
                doc.name = name.clone();
                return Ok(name);
            } else {
                // E.g. "ACC-SINV-" without hashes -> default 5 hashes
                let template = format!("{series_val}.#####");
                let name = Self::parse_series_template(&template, year, month, day, store);
                doc.name = name.clone();
                return Ok(name);
            }
        }

        // 3. Fallback default pattern: DOCTYPE-.YYYY.-.#####
        let clean_dt = schema.name.to_uppercase().replace(' ', "-");
        let fallback_template = format!("{clean_dt}-.YYYY.-.#####");
        let name = Self::parse_series_template(&fallback_template, year, month, day, store);
        doc.name = name.clone();
        Ok(name)
    }

    /// Evaluates dynamic series formatting tokens and counter width.
    #[must_use]
    pub fn parse_series_template(
        template: &str,
        year: i32,
        month: u32,
        day: u32,
        store: &NamingSeriesStore,
    ) -> String {
        // Replace date tokens
        let mut prefix = template
            .replace(".YYYY.", &format!("{year:04}"))
            .replace(".YY.", &format!("{:02}", year % 100))
            .replace(".MM.", &format!("{month:02}"))
            .replace(".DD.", &format!("{day:02}"));

        // Find counter hashes: .##### or .#### or .###
        let mut hash_count = 0;
        if let Some(hash_start) = prefix.find(".#") {
            let hashes = prefix[hash_start + 1..]
                .chars()
                .take_while(|&c| c == '#')
                .count();
            hash_count = hashes;
            let hash_token = format!(".{}", "#".repeat(hashes));
            prefix = prefix.replace(&hash_token, "");
        } else if let Some(hash_start) = prefix.find('#') {
            let hashes = prefix[hash_start..]
                .chars()
                .take_while(|&c| c == '#')
                .count();
            hash_count = hashes;
            let hash_token = "#".repeat(hashes);
            prefix = prefix.replace(&hash_token, "");
        }

        if hash_count == 0 {
            hash_count = 5;
        }

        let seq = store.get_next(&prefix);
        format!("{prefix}{seq:0width$}", width = hash_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_template_parsing_and_increment() {
        let store = NamingSeriesStore::new();
        let template = "ACC-INV-.YYYY.-.MM.-.#####";

        let name1 = NamingEngine::parse_series_template(template, 2026, 10, 16, &store);
        let name2 = NamingEngine::parse_series_template(template, 2026, 10, 16, &store);
        let name3 = NamingEngine::parse_series_template(template, 2026, 10, 16, &store);

        assert_eq!(name1, "ACC-INV-2026-10-00001");
        assert_eq!(name2, "ACC-INV-2026-10-00002");
        assert_eq!(name3, "ACC-INV-2026-10-00003");
    }

    #[test]
    fn test_autoname_field_link() {
        let store = NamingSeriesStore::new();
        let schema = DocTypeSchema {
            name: "Customer".into(),
            module: "CRM".into(),
            is_submittable: false,
            is_single: false,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: Some("field:customer_name".into()),
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            title_field: None,
            search_fields: None,
            image_field: None,
            fields: vec![],
            permissions: vec![],
        };

        let mut doc = Document::new(
            "Customer",
            serde_json::json!({
                "customer_name": "Apex Global Logistics"
            }),
        );

        let name = NamingEngine::autoname(&mut doc, &schema, &store).unwrap();
        assert_eq!(name, "Apex Global Logistics");
        assert_eq!(doc.name, "Apex Global Logistics");
    }
}
