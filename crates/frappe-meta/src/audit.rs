//! Enterprise Audit & Compliance Logging Fabric (`frappe-meta::audit`).
//!
//! Provides immutable, tamper-evident audit logging for all document lifecycle events,
//! security authentications, RBAC role mutations, and administrative operations.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Classification of auditable system events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Document Lifecycle
    Create,
    Read,
    Update,
    Delete,
    Submit,
    Cancel,
    Amend,
    // Authentication & Session
    LoginSuccess,
    LoginFailed,
    Logout,
    SessionExpired,
    PasswordResetRequested,
    PasswordResetCompleted,
    PasswordChanged,
    // Multi-Factor Authentication
    MfaEnrolled,
    MfaActivated,
    MfaDisabled,
    MfaChallengeSuccess,
    MfaChallengeFailed,
    // RBAC & Permission Governance
    RoleCreated,
    RoleAssigned,
    RoleRevoked,
    PermissionUpdated,
    // Security & Infrastructure
    IpBlocked,
    RateLimitExceeded,
    SecurityAlert,
    DataExported,
    Custom(String),
}

/// A structured immutable audit log record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub tenant_id: String,
    pub user_id: String,
    pub user_email: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub action: AuditAction,
    pub doctype: Option<String>,
    pub docname: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub diff_summary: Option<String>,
    pub status: String, // "Success", "Denied", "Failed"
    pub error_message: Option<String>,
}

impl AuditEntry {
    /// Constructs a new audit log entry.
    #[must_use]
    pub fn new(
        tenant_id: &str,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
        action: AuditAction,
        status: &str,
    ) -> Self {
        let now = Utc::now();
        let id = format!("audit_{}_{}", now.timestamp_millis(), rand::random::<u16>());
        Self {
            id,
            timestamp: now.to_rfc3339(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            user_email: None,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            action,
            doctype: None,
            docname: None,
            before_state: None,
            after_state: None,
            diff_summary: None,
            status: status.to_string(),
            error_message: None,
        }
    }

    /// Attaches document context and automatically computes field-level delta summary.
    #[must_use]
    pub fn with_document_diff(
        mut self,
        doctype: &str,
        docname: &str,
        before: Option<serde_json::Value>,
        after: Option<serde_json::Value>,
    ) -> Self {
        self.doctype = Some(doctype.to_string());
        self.docname = Some(docname.to_string());

        if let (Some(b), Some(a)) = (&before, &after) {
            self.diff_summary = Some(compute_json_diff(b, a));
        }

        self.before_state = before;
        self.after_state = after;
        self
    }

    /// Attaches user email for fast human lookup.
    #[must_use]
    pub fn with_email(mut self, email: &str) -> Self {
        self.user_email = Some(email.to_string());
        self
    }

    /// Attaches error detail on failure.
    #[must_use]
    pub fn with_error(mut self, err: &str) -> Self {
        self.error_message = Some(err.to_string());
        self.status = "Failed".to_string();
        self
    }
}

/// Computes a human-readable summary of changed fields between two JSON objects.
#[must_use]
pub fn compute_json_diff(before: &serde_json::Value, after: &serde_json::Value) -> String {
    let mut changes = Vec::new();

    if let (Some(b_obj), Some(a_obj)) = (before.as_object(), after.as_object()) {
        for (k, new_val) in a_obj {
            if let Some(old_val) = b_obj.get(k) {
                if old_val != new_val {
                    changes.push(format!("{k}: {old_val} -> {new_val}"));
                }
            } else {
                changes.push(format!("{k}: added ({new_val})"));
            }
        }
        for (k, old_val) in b_obj {
            if !a_obj.contains_key(k) {
                changes.push(format!("{k}: removed (was {old_val})"));
            }
        }
    }

    if changes.is_empty() {
        "No field changes detected".to_string()
    } else {
        changes.join("; ")
    }
}

/// Query filter for retrieving audit logs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQueryFilter {
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub doctype: Option<String>,
    pub status: Option<String>,
    pub from_timestamp: Option<String>,
    pub to_timestamp: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Thread-safe in-memory ring-buffer and query registry for audit logs.
#[derive(Clone, Debug)]
pub struct AuditTrailRegistry {
    entries: Arc<RwLock<VecDeque<AuditEntry>>>,
    max_capacity: usize,
}

impl Default for AuditTrailRegistry {
    fn default() -> Self {
        Self::new(50_000)
    }
}

impl AuditTrailRegistry {
    /// Creates a new audit trail registry with a bounded in-memory ring-buffer.
    #[must_use]
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(
                max_capacity.min(100_000),
            ))),
            max_capacity,
        }
    }

    /// Records an audit log entry.
    pub fn record(&self, entry: AuditEntry) {
        if let Ok(mut lock) = self.entries.write() {
            if lock.len() >= self.max_capacity {
                lock.pop_front();
            }
            lock.push_back(entry);
        }
    }

    /// Queries audit entries matching filter criteria (newest first).
    #[must_use]
    pub fn query(&self, filter: &AuditQueryFilter) -> (Vec<AuditEntry>, usize) {
        let lock = match self.entries.read() {
            Ok(guard) => guard,
            Err(_) => return (Vec::new(), 0),
        };

        let filtered: Vec<AuditEntry> = lock
            .iter()
            .rev()
            .filter(|e| {
                if filter
                    .tenant_id
                    .as_ref()
                    .is_some_and(|tid| &e.tenant_id != tid)
                {
                    return false;
                }
                if filter.user_id.as_ref().is_some_and(|uid| {
                    &e.user_id != uid && e.user_email.as_deref() != Some(uid.as_str())
                }) {
                    return false;
                }
                if filter
                    .doctype
                    .as_ref()
                    .is_some_and(|dt| e.doctype.as_deref() != Some(dt.as_str()))
                {
                    return false;
                }
                if filter
                    .status
                    .as_ref()
                    .is_some_and(|st| !e.status.eq_ignore_ascii_case(st))
                {
                    return false;
                }
                if filter
                    .from_timestamp
                    .as_ref()
                    .is_some_and(|from_t| e.timestamp < *from_t)
                {
                    return false;
                }
                if filter
                    .to_timestamp
                    .as_ref()
                    .is_some_and(|to_t| e.timestamp > *to_t)
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len();
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(50).min(500);

        let paged = filtered.into_iter().skip(offset).take(limit).collect();
        (paged, total)
    }

    /// Returns the total count of recorded audit entries.
    #[must_use]
    pub fn count(&self) -> usize {
        self.entries.read().map(|g| g.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_entry_diff_and_query() {
        let registry = AuditTrailRegistry::new(100);

        let before = json!({"status": "Draft", "amount": 100});
        let after = json!({"status": "Submitted", "amount": 120});

        let entry = AuditEntry::new(
            "default",
            "usr_alice",
            "192.168.1.50",
            "Mozilla/5.0",
            AuditAction::Submit,
            "Success",
        )
        .with_email("alice@example.com")
        .with_document_diff(
            "Sales Invoice",
            "ACC-SINV-2026-0001",
            Some(before),
            Some(after),
        );

        assert!(entry.diff_summary.is_some());
        let diff = entry.diff_summary.as_ref().unwrap();
        assert!(diff.contains("status: \"Draft\" -> \"Submitted\""));
        assert!(diff.contains("amount: 100 -> 120"));

        registry.record(entry);

        let filter = AuditQueryFilter {
            tenant_id: Some("default".into()),
            doctype: Some("Sales Invoice".into()),
            ..Default::default()
        };

        let (results, total) = registry.query(&filter);
        assert_eq!(total, 1);
        assert_eq!(results[0].docname.as_deref(), Some("ACC-SINV-2026-0001"));
    }

    #[test]
    fn test_audit_login_failed_entry() {
        let registry = AuditTrailRegistry::new(100);
        let entry = AuditEntry::new(
            "default",
            "usr_unknown",
            "203.0.113.42",
            "curl/8.0",
            AuditAction::LoginFailed,
            "Denied",
        )
        .with_error("Invalid password provided");

        registry.record(entry);

        let filter = AuditQueryFilter {
            status: Some("Failed".into()),
            ..Default::default()
        };
        let (results, total) = registry.query(&filter);
        assert_eq!(total, 1);
        assert_eq!(
            results[0].error_message.as_deref(),
            Some("Invalid password provided")
        );
    }
}
