//! ERPNext Clean-Room Plugin SDK for Rust & Dioxus (WASM-only).
//!
//! Provides the primary interfaces, context types, and execution contracts for
//! authoring sandboxed WASM components (`wasm32-wasip2`).

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::{
        DocEvent, DocMutationResult, DocRecord, HookError, PluginCapabilityManifest, PluginContext,
        PluginManifest,
    };
}

/// Represents an active ERP document record passed to a plugin hook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocRecord {
    pub doctype: CompactString,
    pub name: CompactString,
    pub data: serde_json::Value,
    pub is_new: bool,
}

/// Result of a plugin document event hook execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocMutationResult {
    /// Allow the document transition to proceed without changes.
    Proceed,
    /// Amend document fields with a partial JSON payload.
    Amend(serde_json::Value),
    /// Abort the transition and reject the transaction with an error message.
    Abort(CompactString),
}

/// Strongly typed errors encountered during plugin hook execution.
#[derive(Debug, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum HookError {
    #[error("Validation failed: {0}")]
    ValidationError(CompactString),
    #[error("Authorization denied: {0}")]
    PermissionDenied(CompactString),
    #[error("Host communication error: {0}")]
    InternalError(CompactString),
}

/// Document lifecycle event triggers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocEvent {
    BeforeInsert,
    Validate,
    BeforeSubmit,
    AfterSubmit,
    OnCancel,
    OnTrash,
}

/// Sandboxed plugin execution capability manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCapabilityManifest {
    pub plugin_id: CompactString,
    pub version: CompactString,
    pub storage_whitelist: Vec<CompactString>,
    pub network_egress_whitelist: Vec<CompactString>,
    pub max_memory_mb: u32,
    pub max_fuel_instructions: u64,
}

impl Default for PluginCapabilityManifest {
    fn default() -> Self {
        Self {
            plugin_id: "unnamed-plugin".into(),
            version: "0.1.0".into(),
            storage_whitelist: vec!["Item".into(), "Customer".into()],
            network_egress_whitelist: vec![],
            max_memory_mb: 64,
            max_fuel_instructions: 10_000_000,
        }
    }
}

/// Plugin package manifest descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    pub name: CompactString,
    pub title: CompactString,
    pub author: CompactString,
    pub license: CompactString,
    pub capabilities: PluginCapabilityManifest,
}

/// Execution context provided by the host to every plugin invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginContext {
    pub tenant_id: CompactString,
    pub user_id: CompactString,
    pub user_roles: Vec<CompactString>,
    pub timestamp_utc: i64,
}

impl PluginContext {
    /// Creates a new plugin execution context.
    #[must_use]
    pub fn new(tenant_id: &str, user_id: &str, user_roles: Vec<CompactString>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            user_id: user_id.into(),
            user_roles,
            timestamp_utc: chrono::Utc::now().timestamp(),
        }
    }

    /// Checks whether the executing user holds a specific role.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.user_roles.iter().any(|r| r.as_str() == role)
    }

    /// Queries host documents safely within permitted manifest capabilities.
    pub fn query_doc(
        &self,
        doctype: &str,
        _filters: &serde_json::Value,
    ) -> Result<Vec<DocRecord>, HookError> {
        if doctype.is_empty() {
            return Err(HookError::ValidationError("Doctype cannot be empty".into()));
        }
        // In native / test mode, returns empty slice or simulated host responses
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_context_role_check() {
        let ctx = PluginContext::new(
            "tenant_a",
            "user_1",
            vec!["Sales User".into(), "Manager".into()],
        );
        assert!(ctx.has_role("Manager"));
        assert!(!ctx.has_role("System Manager"));
    }

    #[test]
    fn test_doc_mutation_result_amend_and_abort() {
        let doc = DocRecord {
            doctype: "Sales Order".into(),
            name: "SO-2026-001".into(),
            data: serde_json::json!({
                "grand_total": 12500.0,
                "customer": "Apex Global"
            }),
            is_new: true,
        };

        // Simulated hook logic
        let grand_total = doc
            .data
            .get("grand_total")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let mutation = if grand_total > 10_000.0 {
            DocMutationResult::Amend(serde_json::json!({ "discount_percentage": 5.0 }))
        } else {
            DocMutationResult::Proceed
        };

        match mutation {
            DocMutationResult::Amend(val) => {
                assert_eq!(val.get("discount_percentage").unwrap().as_f64(), Some(5.0));
            }
            _ => panic!("Expected amend result"),
        }
    }

    #[test]
    fn test_plugin_capability_manifest_defaults() {
        let manifest = PluginCapabilityManifest::default();
        assert_eq!(manifest.max_memory_mb, 64);
        assert_eq!(manifest.max_fuel_instructions, 10_000_000);
        assert!(manifest.storage_whitelist.contains(&"Item".into()));
    }
}
