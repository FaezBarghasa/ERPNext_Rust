//! Incremental Schema & Visual AST Migration Engine (`doctype_diff`).
//!
//! Provides Phase 10:
//! - Non-destructive schema diffing between Desktop Studio state and Live VPS state.
//! - Non-destructive SurrealQL DDL migration generation (`DEFINE TABLE`, `DEFINE FIELD`).
//! - Visual page AST modification tracking (Canvas block tree revisions).
//! - Cryptographic patch bundle synthesis (`UpgradePatchBundle`) with SHA-256 integrity validation.

use chrono::Utc;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Snapshot of a single DocField within a DocType.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldSnapshot {
    pub name: CompactString,
    pub fieldtype: CompactString,
    pub reqd: bool,
    pub unique: bool,
    pub options: Option<CompactString>,
    pub default_value: Option<String>,
}

impl FieldSnapshot {
    #[must_use]
    pub fn new(name: impl Into<CompactString>, fieldtype: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            fieldtype: fieldtype.into(),
            reqd: false,
            unique: false,
            options: None,
            default_value: None,
        }
    }

    #[must_use]
    pub fn required(mut self) -> Self {
        self.reqd = true;
        self
    }

    #[must_use]
    pub fn unique_field(mut self) -> Self {
        self.unique = true;
        self
    }
}

/// Snapshot of a complete DocType schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocTypeSnapshot {
    pub name: CompactString,
    pub module: CompactString,
    pub is_submittable: bool,
    pub is_child_table: bool,
    pub fields: Vec<FieldSnapshot>,
}

impl DocTypeSnapshot {
    #[must_use]
    pub fn new(name: impl Into<CompactString>, module: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            module: module.into(),
            is_submittable: false,
            is_child_table: false,
            fields: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_field(mut self, field: FieldSnapshot) -> Self {
        self.fields.push(field);
        self
    }

    #[must_use]
    pub fn with_submittable(mut self) -> Self {
        self.is_submittable = true;
        self
    }
}

/// Nature of modification to a schema field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldDiffKind {
    Added,
    Modified,
    Removed,
}

/// Detailed difference for an individual field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDiff {
    pub fieldname: CompactString,
    pub kind: FieldDiffKind,
    pub old_field: Option<FieldSnapshot>,
    pub new_field: Option<FieldSnapshot>,
    pub is_destructive: bool,
}

/// Nature of modification to a DocType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocTypeDiffKind {
    Created,
    Altered,
    Deleted,
}

/// Detailed difference across an entire DocType.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocTypeDiff {
    pub doctype_name: CompactString,
    pub kind: DocTypeDiffKind,
    pub field_diffs: Vec<FieldDiff>,
    pub is_destructive: bool,
}

/// Nature of change to a visual page or canvas block tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageDiffKind {
    Created,
    Updated,
    Deleted,
}

/// Difference in visual canvas block hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualPageDiff {
    pub slug: CompactString,
    pub title: CompactString,
    pub kind: PageDiffKind,
    pub block_count_old: usize,
    pub block_count_new: usize,
}

/// Client script modification item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomScriptDiff {
    pub doctype: CompactString,
    pub kind: PageDiffKind,
    pub script_content: String,
}

/// Atomic, signed upgrade patch bundle transmitted from Desktop to VPS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpgradePatchBundle {
    pub patch_id: CompactString,
    pub source_version: CompactString,
    pub target_version: CompactString,
    pub created_at: CompactString,
    pub doctype_diffs: Vec<DocTypeDiff>,
    pub page_diffs: Vec<VisualPageDiff>,
    pub custom_scripts: Vec<CustomScriptDiff>,
    pub surrealql_statements: Vec<String>,
    pub is_destructive: bool,
    pub checksum_sha256: CompactString,
}

impl UpgradePatchBundle {
    /// Validates the cryptographic SHA-256 integrity checksum of the patch.
    #[must_use]
    pub fn verify_checksum(&self) -> bool {
        DocTypeDiffEngine::verify_patch_checksum(self)
    }
}

/// Core Schema & AST Diffing Engine.
pub struct DocTypeDiffEngine;

impl DocTypeDiffEngine {
    /// Compares two sets of DocType schemas and returns structural differences.
    #[must_use]
    pub fn diff_doctypes(
        local: &[DocTypeSnapshot],
        remote: &[DocTypeSnapshot],
    ) -> Vec<DocTypeDiff> {
        let mut diffs = Vec::new();
        let remote_map: HashMap<&str, &DocTypeSnapshot> =
            remote.iter().map(|d| (d.name.as_str(), d)).collect();
        let local_map: HashMap<&str, &DocTypeSnapshot> =
            local.iter().map(|d| (d.name.as_str(), d)).collect();

        // 1. Detect Created or Altered DocTypes
        for local_dt in local {
            match remote_map.get(local_dt.name.as_str()) {
                None => {
                    // Entirely new DocType created locally
                    let field_diffs = local_dt
                        .fields
                        .iter()
                        .map(|f| FieldDiff {
                            fieldname: f.name.clone(),
                            kind: FieldDiffKind::Added,
                            old_field: None,
                            new_field: Some(f.clone()),
                            is_destructive: false,
                        })
                        .collect();

                    diffs.push(DocTypeDiff {
                        doctype_name: local_dt.name.clone(),
                        kind: DocTypeDiffKind::Created,
                        field_diffs,
                        is_destructive: false,
                    });
                }
                Some(remote_dt) => {
                    // DocType exists on both sides: compare fields
                    let mut field_diffs = Vec::new();
                    let remote_fields: HashMap<&str, &FieldSnapshot> = remote_dt
                        .fields
                        .iter()
                        .map(|f| (f.name.as_str(), f))
                        .collect();
                    let local_fields: HashMap<&str, &FieldSnapshot> = local_dt
                        .fields
                        .iter()
                        .map(|f| (f.name.as_str(), f))
                        .collect();

                    // Added or Modified fields
                    for local_f in &local_dt.fields {
                        match remote_fields.get(local_f.name.as_str()) {
                            None => {
                                field_diffs.push(FieldDiff {
                                    fieldname: local_f.name.clone(),
                                    kind: FieldDiffKind::Added,
                                    old_field: None,
                                    new_field: Some(local_f.clone()),
                                    is_destructive: false,
                                });
                            }
                            Some(&remote_f) => {
                                if remote_f != local_f {
                                    // Check if type changed (potentially destructive)
                                    let type_changed = remote_f.fieldtype != local_f.fieldtype;
                                    field_diffs.push(FieldDiff {
                                        fieldname: local_f.name.clone(),
                                        kind: FieldDiffKind::Modified,
                                        old_field: Some((*remote_f).clone()),
                                        new_field: Some(local_f.clone()),
                                        is_destructive: type_changed,
                                    });
                                }
                            }
                        }
                    }

                    // Removed fields
                    for remote_f in &remote_dt.fields {
                        if !local_fields.contains_key(remote_f.name.as_str()) {
                            field_diffs.push(FieldDiff {
                                fieldname: remote_f.name.clone(),
                                kind: FieldDiffKind::Removed,
                                old_field: Some(remote_f.clone()),
                                new_field: None,
                                is_destructive: true,
                            });
                        }
                    }

                    if !field_diffs.is_empty() {
                        let has_destructive = field_diffs.iter().any(|f| f.is_destructive);
                        diffs.push(DocTypeDiff {
                            doctype_name: local_dt.name.clone(),
                            kind: DocTypeDiffKind::Altered,
                            field_diffs,
                            is_destructive: has_destructive,
                        });
                    }
                }
            }
        }

        // 2. Detect Deleted DocTypes (present remotely, missing locally)
        for remote_dt in remote {
            if !local_map.contains_key(remote_dt.name.as_str()) {
                diffs.push(DocTypeDiff {
                    doctype_name: remote_dt.name.clone(),
                    kind: DocTypeDiffKind::Deleted,
                    field_diffs: Vec::new(),
                    is_destructive: true,
                });
            }
        }

        diffs
    }

    /// Compares local visual page models against remote VPS pages.
    #[must_use]
    pub fn diff_pages(
        local: &[(CompactString, CompactString, usize)], // (slug, title, block_count)
        remote: &[(CompactString, CompactString, usize)],
    ) -> Vec<VisualPageDiff> {
        let mut diffs = Vec::new();
        let remote_map: HashMap<&str, (&str, usize)> = remote
            .iter()
            .map(|(s, t, c)| (s.as_str(), (t.as_str(), *c)))
            .collect();
        let local_set: HashSet<&str> = local.iter().map(|(s, _, _)| s.as_str()).collect();

        for (slug, title, block_count) in local {
            match remote_map.get(slug.as_str()) {
                None => {
                    diffs.push(VisualPageDiff {
                        slug: slug.clone(),
                        title: title.clone(),
                        kind: PageDiffKind::Created,
                        block_count_old: 0,
                        block_count_new: *block_count,
                    });
                }
                Some((_, old_count)) => {
                    if old_count != block_count {
                        diffs.push(VisualPageDiff {
                            slug: slug.clone(),
                            title: title.clone(),
                            kind: PageDiffKind::Updated,
                            block_count_old: *old_count,
                            block_count_new: *block_count,
                        });
                    }
                }
            }
        }

        for (slug, title, count) in remote {
            if !local_set.contains(slug.as_str()) {
                diffs.push(VisualPageDiff {
                    slug: slug.clone(),
                    title: title.clone(),
                    kind: PageDiffKind::Deleted,
                    block_count_old: *count,
                    block_count_new: 0,
                });
            }
        }

        diffs
    }

    /// Generates non-destructive SurrealQL statements for incremental schema migrations.
    #[must_use]
    pub fn generate_non_destructive_surrealql(diffs: &[DocTypeDiff]) -> Vec<String> {
        let mut statements = Vec::new();

        for diff in diffs {
            let table_name = diff.doctype_name.to_lowercase().replace(' ', "_");

            match diff.kind {
                DocTypeDiffKind::Created => {
                    statements.push(format!("DEFINE TABLE {} SCHEMALESS;", table_name));
                    for f in &diff.field_diffs {
                        if let Some(new_f) = &f.new_field {
                            let sql_type = Self::map_field_to_surrealql(&new_f.fieldtype);
                            let mut stmt = format!(
                                "DEFINE FIELD {} ON TABLE {} TYPE {};",
                                new_f.name.to_lowercase().replace(' ', "_"),
                                table_name,
                                sql_type
                            );
                            if new_f.unique {
                                stmt.push_str(&format!(
                                    "\nDEFINE INDEX idx_{}_{} ON TABLE {} COLUMNS {} UNIQUE;",
                                    table_name,
                                    new_f.name.to_lowercase().replace(' ', "_"),
                                    table_name,
                                    new_f.name.to_lowercase().replace(' ', "_")
                                ));
                            }
                            statements.push(stmt);
                        }
                    }
                }
                DocTypeDiffKind::Altered => {
                    for f in &diff.field_diffs {
                        if let (FieldDiffKind::Added, Some(new_f)) = (f.kind, &f.new_field) {
                            let sql_type = Self::map_field_to_surrealql(&new_f.fieldtype);
                            statements.push(format!(
                                "DEFINE FIELD {} ON TABLE {} TYPE {};",
                                new_f.name.to_lowercase().replace(' ', "_"),
                                table_name,
                                sql_type
                            ));
                        }
                    }
                }
                DocTypeDiffKind::Deleted => {
                    // Do NOT drop tables in non-destructive mode; preserve existing live customer data
                }
            }
        }

        statements
    }

    /// Maps ERP field types to SurrealDB QL types.
    #[must_use]
    pub fn map_field_to_surrealql(fieldtype: &str) -> &'static str {
        match fieldtype.to_ascii_lowercase().as_str() {
            "int" | "check" => "int",
            "currency" | "float" | "percent" => "decimal",
            "date" | "datetime" | "time" => "datetime",
            "table" | "table multiselect" => "array",
            "json" | "code" => "object",
            _ => "string",
        }
    }

    /// Synthesizes a signed UpgradePatchBundle from computed diffs.
    #[must_use]
    pub fn build_upgrade_patch(
        source_ver: &str,
        target_ver: &str,
        local_doctypes: &[DocTypeSnapshot],
        remote_doctypes: &[DocTypeSnapshot],
        local_pages: &[(CompactString, CompactString, usize)],
        remote_pages: &[(CompactString, CompactString, usize)],
    ) -> UpgradePatchBundle {
        let doctype_diffs = Self::diff_doctypes(local_doctypes, remote_doctypes);
        let page_diffs = Self::diff_pages(local_pages, remote_pages);
        let surrealql = Self::generate_non_destructive_surrealql(&doctype_diffs);
        let is_destructive = doctype_diffs.iter().any(|d| d.is_destructive);

        let nanos = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| Utc::now().timestamp_micros());
        let patch_id = CompactString::from(format!("patch_{}", nanos));

        let created_at = CompactString::from(Utc::now().to_rfc3339());

        let mut bundle = UpgradePatchBundle {
            patch_id,
            source_version: CompactString::from(source_ver),
            target_version: CompactString::from(target_ver),
            created_at,
            doctype_diffs,
            page_diffs,
            custom_scripts: Vec::new(),
            surrealql_statements: surrealql,
            is_destructive,
            checksum_sha256: CompactString::default(),
        };

        let checksum = Self::compute_patch_checksum(&bundle);
        bundle.checksum_sha256 = CompactString::from(checksum);

        bundle
    }

    /// Computes the cryptographic SHA-256 hash of the patch bundle content.
    #[must_use]
    pub fn compute_patch_checksum(bundle: &UpgradePatchBundle) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bundle.patch_id.as_bytes());
        hasher.update(bundle.source_version.as_bytes());
        hasher.update(bundle.target_version.as_bytes());
        hasher.update(bundle.created_at.as_bytes());

        if let Ok(serialized_dt) = serde_json::to_vec(&bundle.doctype_diffs) {
            hasher.update(&serialized_dt);
        }
        if let Ok(serialized_pg) = serde_json::to_vec(&bundle.page_diffs) {
            hasher.update(&serialized_pg);
        }
        if let Ok(serialized_sql) = serde_json::to_vec(&bundle.surrealql_statements) {
            hasher.update(&serialized_sql);
        }

        hex::encode(hasher.finalize())
    }

    /// Validates that a received patch bundle's checksum matches its contents.
    #[must_use]
    pub fn verify_patch_checksum(bundle: &UpgradePatchBundle) -> bool {
        let expected = Self::compute_patch_checksum(bundle);
        bundle.checksum_sha256.as_str() == expected.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctype_diff_created_and_altered() {
        let remote_doctype = DocTypeSnapshot::new("Item", "Stock")
            .with_field(FieldSnapshot::new("item_code", "Data").required())
            .with_field(FieldSnapshot::new("item_name", "Data"));

        let local_doctype = DocTypeSnapshot::new("Item", "Stock")
            .with_field(FieldSnapshot::new("item_code", "Data").required())
            .with_field(FieldSnapshot::new("item_name", "Data"))
            .with_field(FieldSnapshot::new("weight", "Float")); // Added field

        let new_doctype = DocTypeSnapshot::new("Warehouse_Zone", "Stock")
            .with_field(FieldSnapshot::new("zone_code", "Data").required());

        let diffs =
            DocTypeDiffEngine::diff_doctypes(&[local_doctype, new_doctype], &[remote_doctype]);

        assert_eq!(diffs.len(), 2);

        let item_diff = diffs.iter().find(|d| d.doctype_name == "Item").unwrap();
        assert_eq!(item_diff.kind, DocTypeDiffKind::Altered);
        assert_eq!(item_diff.field_diffs.len(), 1);
        assert_eq!(item_diff.field_diffs[0].fieldname, "weight");
        assert_eq!(item_diff.field_diffs[0].kind, FieldDiffKind::Added);
        assert!(!item_diff.is_destructive);

        let zone_diff = diffs
            .iter()
            .find(|d| d.doctype_name == "Warehouse_Zone")
            .unwrap();
        assert_eq!(zone_diff.kind, DocTypeDiffKind::Created);
        assert!(!zone_diff.is_destructive);
    }

    #[test]
    fn test_destructive_field_removal_detected() {
        let remote_doctype = DocTypeSnapshot::new("Customer", "CRM")
            .with_field(FieldSnapshot::new("customer_name", "Data"))
            .with_field(FieldSnapshot::new("tax_id", "Data"));

        // Local dropped "tax_id"
        let local_doctype = DocTypeSnapshot::new("Customer", "CRM")
            .with_field(FieldSnapshot::new("customer_name", "Data"));

        let diffs = DocTypeDiffEngine::diff_doctypes(&[local_doctype], &[remote_doctype]);
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].is_destructive);
        assert_eq!(diffs[0].field_diffs[0].kind, FieldDiffKind::Removed);
    }

    #[test]
    fn test_non_destructive_surrealql_generation() {
        let remote_doctype = DocTypeSnapshot::new("Item", "Stock")
            .with_field(FieldSnapshot::new("item_code", "Data"));

        let local_doctype = DocTypeSnapshot::new("Item", "Stock")
            .with_field(FieldSnapshot::new("item_code", "Data"))
            .with_field(FieldSnapshot::new("standard_rate", "Currency"));

        let diffs = DocTypeDiffEngine::diff_doctypes(&[local_doctype], &[remote_doctype]);
        let sql = DocTypeDiffEngine::generate_non_destructive_surrealql(&diffs);

        assert_eq!(sql.len(), 1);
        assert_eq!(
            sql[0],
            "DEFINE FIELD standard_rate ON TABLE item TYPE decimal;"
        );
    }

    #[test]
    fn test_page_diff_tracking() {
        let remote_pages = vec![
            (CompactString::from("home"), CompactString::from("Home"), 5),
            (
                CompactString::from("old_pricing"),
                CompactString::from("Old Pricing"),
                3,
            ),
        ];

        let local_pages = vec![
            (CompactString::from("home"), CompactString::from("Home"), 8), // Updated
            (
                CompactString::from("new_about"),
                CompactString::from("About"),
                4,
            ), // Created
        ];

        let diffs = DocTypeDiffEngine::diff_pages(&local_pages, &remote_pages);
        assert_eq!(diffs.len(), 3);

        let home_diff = diffs.iter().find(|d| d.slug == "home").unwrap();
        assert_eq!(home_diff.kind, PageDiffKind::Updated);
        assert_eq!(home_diff.block_count_old, 5);
        assert_eq!(home_diff.block_count_new, 8);

        let about_diff = diffs.iter().find(|d| d.slug == "new_about").unwrap();
        assert_eq!(about_diff.kind, PageDiffKind::Created);

        let del_diff = diffs.iter().find(|d| d.slug == "old_pricing").unwrap();
        assert_eq!(del_diff.kind, PageDiffKind::Deleted);
    }

    #[test]
    fn test_upgrade_patch_bundle_integrity_checksum() {
        let remote = vec![DocTypeSnapshot::new("Item", "Stock")];
        let local = vec![
            DocTypeSnapshot::new("Item", "Stock").with_field(FieldSnapshot::new("barcode", "Data")),
        ];

        let bundle =
            DocTypeDiffEngine::build_upgrade_patch("0.3.0", "0.3.1", &local, &remote, &[], &[]);

        assert!(DocTypeDiffEngine::verify_patch_checksum(&bundle));
        assert!(!bundle.checksum_sha256.is_empty());

        // Tampering should fail validation
        let mut tampered = bundle.clone();
        tampered.target_version = "0.9.9".into();
        assert!(!DocTypeDiffEngine::verify_patch_checksum(&tampered));
    }
}
