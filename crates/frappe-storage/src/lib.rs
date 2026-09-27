//! Tenant context, queue states, CAS deduplication, and SurrealDB storage backend.
pub mod drive;
pub mod surreal;

pub use drive::{DeduplicatedStorage, DriveFile, DriveFolder, StorageError};
pub use surreal::open_tenant;

use std::collections::HashMap;
use std::sync::RwLock;

/// Contextual tenant information attached to requests.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub site_name: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

impl TenantContext {
    /// Generates SurrealQL USE statement.
    #[must_use]
    pub fn surreal_use(&self) -> String {
        format!("USE NS {} DB {};", self.tenant_id, self.site_name)
    }

    /// Resolves tenant context from HTTP headers.
    #[must_use]
    pub fn from_headers(headers: &HashMap<String, String>, host: &str) -> Option<Self> {
        let site = headers
            .get("X-Frappe-Site-Name")
            .cloned()
            .or_else(|| host.split('.').next().map(ToString::to_string))
            .filter(|s| !s.is_empty())?;
        Some(Self {
            tenant_id: format!("tenant_{}", site.replace('-', "_")),
            site_name: site,
            user_id: "guest".into(),
            roles: vec!["Guest".into()],
        })
    }
}

/// In-memory registry of active tenants.
#[derive(Default)]
pub struct TenantRegistry {
    inner: RwLock<HashMap<String, TenantContext>>,
}

impl TenantRegistry {
    pub fn insert(&self, ctx: TenantContext) {
        if let Ok(mut lock) = self.inner.write() {
            lock.insert(ctx.site_name.clone(), ctx);
        }
    }

    #[must_use]
    pub fn get(&self, site: &str) -> Option<TenantContext> {
        self.inner.read().ok()?.get(site).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct QueueTask {
    pub id: String,
    pub queue: &'static str,
    pub state: TaskState,
}

impl QueueTask {
    pub fn transition(&mut self, to: TaskState) -> bool {
        let ok = matches!(
            (self.state, to),
            (TaskState::Queued, TaskState::Processing)
                | (TaskState::Processing, TaskState::Completed)
                | (TaskState::Processing, TaskState::Failed)
                | (TaskState::Failed, TaskState::Queued)
        );
        if ok {
            self.state = to;
        }
        ok
    }
}

/// Live-query mutation diff pushed over WebSocket.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiveDiff {
    pub table: String,
    pub id: String,
    pub op: String,
    pub payload_json: String,
}

impl LiveDiff {
    #[must_use]
    pub fn to_frame(&self) -> String {
        format!("{}:{}:{}:{}", self.table, self.id, self.op, self.payload_json)
    }
}

/// Helper function to generate SurrealQL RELATE graph edge statement.
#[must_use]
pub fn relate(a: &str, verb: &str, b: &str, attrs: &str) -> String {
    if attrs.is_empty() {
        format!("RELATE {a}->{verb}->{b};")
    } else {
        format!("RELATE {a}->{verb}->{b} SET {attrs};")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_storage() {
        let storage = DeduplicatedStorage::new();
        let payload = b"Hello, World! Common invoice template bytes";

        let (file1, is_dup1) = storage
            .store_file(
                "file_1".into(),
                "invoice_template.html".into(),
                None,
                "text/html".into(),
                "admin".into(),
                payload,
            )
            .unwrap();
        assert!(!is_dup1);
        assert_eq!(storage.physical_payload_count(), 1);

        let (file2, is_dup2) = storage
            .store_file(
                "file_2".into(),
                "duplicate_invoice.html".into(),
                None,
                "text/html".into(),
                "user2".into(),
                payload,
            )
            .unwrap();
        assert!(is_dup2);
        assert_eq!(storage.physical_payload_count(), 1);
        assert_eq!(file1.content_hash, file2.content_hash);

        let retrieved = storage.read_file("file_2").unwrap();
        assert_eq!(retrieved, payload);
    }
}
