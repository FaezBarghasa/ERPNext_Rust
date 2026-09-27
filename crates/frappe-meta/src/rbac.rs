use serde::{Deserialize, Serialize};

/// Organizational Role definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Role {
    /// Role identifier (e.g. "Accounts User", "System Manager").
    pub name: String,
}

/// User identity representation in the security graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    /// User identifier / email.
    pub id: String,
    /// Direct role assignments.
    pub roles: Vec<String>,
    /// Organization / company scopes.
    pub allowed_companies: Vec<String>,
}

/// Operations subject to RBAC evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Read / View record.
    Read,
    /// Write / Edit record.
    Write,
    /// Create new record.
    Create,
    /// Delete record.
    Delete,
    /// Submit transaction to ledgers.
    Submit,
    /// Cancel submitted transaction.
    Cancel,
    /// Amend cancelled transaction.
    Amend,
    /// Access analytical reports.
    Report,
    /// Export dataset to CSV/Excel.
    Export,
}

/// Graph edge definition: User -> Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasRoleEdge {
    /// User ID.
    pub user_id: String,
    /// Role name.
    pub role: String,
}

/// Graph edge definition: Role -> DocType with granular permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasPermissionEdge {
    /// Role name.
    pub role: String,
    /// Target DocType table.
    pub doctype: String,
    /// Read permission.
    pub p_read: bool,
    /// Write permission.
    pub p_write: bool,
    /// Create permission.
    pub p_create: bool,
    /// Delete permission.
    pub p_delete: bool,
    /// Submit permission.
    pub p_submit: bool,
    /// Cancel permission.
    pub p_cancel: bool,
    /// Amend permission.
    pub p_amend: bool,
    /// Field permission level (0 = document level, 1+ = field level).
    pub permlevel: u8,
}

impl HasPermissionEdge {
    /// Checks if this edge grants the specified permission.
    #[must_use]
    pub fn allows(&self, perm: Permission) -> bool {
        match perm {
            Permission::Read => self.p_read,
            Permission::Write => self.p_write,
            Permission::Create => self.p_create,
            Permission::Delete => self.p_delete,
            Permission::Submit => self.p_submit,
            Permission::Cancel => self.p_cancel,
            Permission::Amend => self.p_amend,
            Permission::Report => self.p_read,
            Permission::Export => self.p_read,
        }
    }
}

/// Evaluates whether a user with given roles is permitted to perform an operation.
#[must_use]
pub fn check_permission(
    user_roles: &[String],
    permission_edges: &[HasPermissionEdge],
    perm: Permission,
    target_permlevel: u8,
) -> bool {
    // System Manager bypass
    if user_roles.iter().any(|r| r == "System Manager" || r == "Administrator") {
        return true;
    }

    permission_edges.iter().any(|edge| {
        user_roles.contains(&edge.role)
            && edge.permlevel <= target_permlevel
            && edge.allows(perm)
    })
}

/// Generates SurrealDB Row-Level Security (RLS) predicate for a table.
#[must_use]
pub fn compile_rls_policy(doctype: &str, roles_with_read: &[String], roles_with_write: &[String]) -> String {
    let read_roles = roles_with_read
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let write_roles = roles_with_write
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "DEFINE TABLE {doctype} SCHEMAFULL PERMISSIONS \
         FOR select WHERE $auth.roles CONTAINSANY [{read_roles}] OR owner = $auth.id, \
         FOR create, update WHERE $auth.roles CONTAINSANY [{write_roles}], \
         FOR delete WHERE $auth.roles CONTAINS \"System Manager\";"
    )
}
