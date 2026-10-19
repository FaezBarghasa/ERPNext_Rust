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
    /// Execute sandboxed plugins / server methods.
    Execute,
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
            Permission::Execute => self.p_write,
        }
    }
}

/// Evaluates whether a user with given roles is permitted to perform an operation.
/// Standard role definitions across the enterprise ecosystem.
pub const ROLE_SYSTEM_MANAGER: &str = "System Manager";
pub const ROLE_ADMINISTRATOR: &str = "Administrator";
pub const ROLE_WORKER_USER: &str = "Worker User";
pub const ROLE_ACCOUNTANT_USER: &str = "Accountant User";
pub const ROLE_MARKETING_ADMIN: &str = "Marketing Admin";
pub const ROLE_CONTENT_CREATOR: &str = "Content Creator";
pub const ROLE_WEBSITE_UPDATER: &str = "Website Updater";
pub const ROLE_WAREHOUSE_MANAGER: &str = "Warehouse Manager";
pub const ROLE_HR_MANAGER: &str = "HR Manager";
pub const ROLE_SALES_USER: &str = "Sales User";
pub const ROLE_PURCHASE_USER: &str = "Purchase User";
pub const ROLE_MANUFACTURING_USER: &str = "Manufacturing User";

/// Complete list of out-of-the-box standard system roles.
pub const STANDARD_ROLES: &[&str] = &[
    ROLE_SYSTEM_MANAGER,
    ROLE_ADMINISTRATOR,
    ROLE_WORKER_USER,
    ROLE_ACCOUNTANT_USER,
    ROLE_MARKETING_ADMIN,
    ROLE_CONTENT_CREATOR,
    ROLE_WEBSITE_UPDATER,
    ROLE_WAREHOUSE_MANAGER,
    ROLE_HR_MANAGER,
    ROLE_SALES_USER,
    ROLE_PURCHASE_USER,
    ROLE_MANUFACTURING_USER,
];

/// Detailed user record containing authentication, state, and assigned roles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRecord {
    /// Unique user identifier / email.
    pub id: String,
    /// Full display name.
    pub full_name: String,
    /// User email address.
    pub email: String,
    /// Whether user account is enabled.
    pub enabled: bool,
    /// Direct role assignments.
    pub roles: Vec<String>,
    /// Organization / company scopes.
    pub allowed_companies: Vec<String>,
    /// Creation timestamp RFC3339.
    pub created_at: String,
}

impl UserRecord {
    /// Create a new user with default enabled state.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        full_name: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            full_name: full_name.into(),
            email: email.into(),
            enabled: true,
            roles: Vec::new(),
            allowed_companies: vec!["default".into()],
            created_at: now,
        }
    }

    /// Assign a role if not already assigned.
    pub fn add_role(&mut self, role: impl Into<String>) {
        let r = role.into();
        if !self.roles.contains(&r) {
            self.roles.push(r);
        }
    }

    /// Revoke a role.
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
    }

    /// Check if user has specific role.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// Dynamic Role-Based Access Control and Permission Matrix.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicRolePermissionRegistry {
    /// Active roles defined in the system.
    pub roles: Vec<Role>,
    /// Active permission edges (Role -> DocType -> granular permissions).
    pub permission_edges: Vec<HasPermissionEdge>,
}

impl DynamicRolePermissionRegistry {
    /// Creates a registry pre-seeded with enterprise standard roles and default permissions.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut reg = Self::default();
        for &r in STANDARD_ROLES {
            reg.roles.push(Role { name: r.into() });
        }

        // 1. Worker User: Floor terminals, Job Card, Time Log, Stock Entry (draft only)
        reg.grant(
            ROLE_WORKER_USER,
            "Job Card",
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WORKER_USER,
            "Timesheet",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WORKER_USER,
            "Stock Entry",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            0,
        );

        // 2. Accountant User: Financial ledgers, Invoices, Journal Entries
        reg.grant(
            ROLE_ACCOUNTANT_USER,
            "Sales Invoice",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );
        reg.grant(
            ROLE_ACCOUNTANT_USER,
            "Purchase Invoice",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );
        reg.grant(
            ROLE_ACCOUNTANT_USER,
            "Journal Entry",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );
        reg.grant(
            ROLE_ACCOUNTANT_USER,
            "GL Entry",
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_ACCOUNTANT_USER,
            "Payment Entry",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );

        // 3. Marketing Admin: Campaigns, Leads, Newsletters
        reg.grant(
            ROLE_MARKETING_ADMIN,
            "Lead",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_MARKETING_ADMIN,
            "Opportunity",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_MARKETING_ADMIN,
            "Campaign",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_MARKETING_ADMIN,
            "Newsletter",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            0,
        );

        // 4. Content Creator: Educational content, lessons, media, blogs
        reg.grant(
            ROLE_CONTENT_CREATOR,
            "Blog Post",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_CONTENT_CREATOR,
            "Course",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_CONTENT_CREATOR,
            "Lesson",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_CONTENT_CREATOR,
            "Media Asset",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );

        // 5. Website Updater: Storefront, Web Pages, Theme Config
        reg.grant(
            ROLE_WEBSITE_UPDATER,
            "Web Page",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WEBSITE_UPDATER,
            "Storefront Theme",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WEBSITE_UPDATER,
            "Catalog Item",
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            0,
        );

        // 6. Warehouse Manager: Warehouses, Items, Stock Ledger, Shipments
        reg.grant(
            ROLE_WAREHOUSE_MANAGER,
            "Item",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WAREHOUSE_MANAGER,
            "Warehouse",
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        );
        reg.grant(
            ROLE_WAREHOUSE_MANAGER,
            "Stock Entry",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );
        reg.grant(
            ROLE_WAREHOUSE_MANAGER,
            "Delivery Note",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );
        reg.grant(
            ROLE_WAREHOUSE_MANAGER,
            "Purchase Receipt",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            0,
        );

        reg
    }

    /// Add a new role dynamically.
    pub fn add_role(&mut self, role_name: impl Into<String>) -> bool {
        let name = role_name.into();
        if self.roles.iter().any(|r| r.name == name) {
            return false;
        }
        self.roles.push(Role { name });
        true
    }

    /// Delete a role and cascade-remove its permissions.
    pub fn delete_role(&mut self, role_name: &str) -> bool {
        let initial_len = self.roles.len();
        self.roles.retain(|r| r.name != role_name);
        self.permission_edges.retain(|e| e.role != role_name);
        self.roles.len() < initial_len
    }

    /// Grant or upsert granular permissions for a role on a DocType.
    #[allow(clippy::too_many_arguments)]
    pub fn grant(
        &mut self,
        role: &str,
        doctype: &str,
        p_read: bool,
        p_write: bool,
        p_create: bool,
        p_delete: bool,
        p_submit: bool,
        p_cancel: bool,
        p_amend: bool,
        permlevel: u8,
    ) {
        if let Some(existing) = self
            .permission_edges
            .iter_mut()
            .find(|e| e.role == role && e.doctype == doctype && e.permlevel == permlevel)
        {
            existing.p_read = p_read;
            existing.p_write = p_write;
            existing.p_create = p_create;
            existing.p_delete = p_delete;
            existing.p_submit = p_submit;
            existing.p_cancel = p_cancel;
            existing.p_amend = p_amend;
        } else {
            self.permission_edges.push(HasPermissionEdge {
                role: role.into(),
                doctype: doctype.into(),
                p_read,
                p_write,
                p_create,
                p_delete,
                p_submit,
                p_cancel,
                p_amend,
                permlevel,
            });
        }
    }

    /// Revoke all permissions for a role on a DocType.
    pub fn revoke(&mut self, role: &str, doctype: &str) {
        self.permission_edges
            .retain(|e| !(e.role == role && e.doctype == doctype));
    }

    /// Computes aggregated effective permissions for a set of roles on a target DocType.
    #[must_use]
    pub fn effective_permissions(
        &self,
        user_roles: &[String],
        doctype: &str,
    ) -> EffectiveDocTypePermissions {
        // System Manager and Administrator bypass all restrictions
        if user_roles
            .iter()
            .any(|r| r == ROLE_SYSTEM_MANAGER || r == ROLE_ADMINISTRATOR)
        {
            return EffectiveDocTypePermissions {
                doctype: doctype.into(),
                can_read: true,
                can_write: true,
                can_create: true,
                can_delete: true,
                can_submit: true,
                can_cancel: true,
                can_amend: true,
                can_report: true,
                can_export: true,
                can_execute: true,
                is_admin_bypass: true,
            };
        }

        let mut res = EffectiveDocTypePermissions {
            doctype: doctype.into(),
            can_read: false,
            can_write: false,
            can_create: false,
            can_delete: false,
            can_submit: false,
            can_cancel: false,
            can_amend: false,
            can_report: false,
            can_export: false,
            can_execute: false,
            is_admin_bypass: false,
        };

        for edge in &self.permission_edges {
            if edge.doctype == doctype && user_roles.contains(&edge.role) {
                if edge.p_read {
                    res.can_read = true;
                    res.can_report = true;
                    res.can_export = true;
                }
                if edge.p_write {
                    res.can_write = true;
                    res.can_execute = true;
                }
                if edge.p_create {
                    res.can_create = true;
                }
                if edge.p_delete {
                    res.can_delete = true;
                }
                if edge.p_submit {
                    res.can_submit = true;
                }
                if edge.p_cancel {
                    res.can_cancel = true;
                }
                if edge.p_amend {
                    res.can_amend = true;
                }
            }
        }

        res
    }
}

/// Calculated effective permissions for a user or role set on a DocType.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveDocTypePermissions {
    pub doctype: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_create: bool,
    pub can_delete: bool,
    pub can_submit: bool,
    pub can_cancel: bool,
    pub can_amend: bool,
    pub can_report: bool,
    pub can_export: bool,
    pub can_execute: bool,
    pub is_admin_bypass: bool,
}

/// Evaluates whether a user with given roles is permitted to perform an operation on a target DocType.
#[must_use]
pub fn check_permission(
    user_roles: &[String],
    permission_edges: &[HasPermissionEdge],
    target_doctype: &str,
    perm: Permission,
    target_permlevel: u8,
) -> bool {
    // System Manager and Administrator bypass
    if user_roles
        .iter()
        .any(|r| r == ROLE_SYSTEM_MANAGER || r == ROLE_ADMINISTRATOR)
    {
        return true;
    }

    permission_edges.iter().any(|edge| {
        edge.doctype.eq_ignore_ascii_case(target_doctype)
            && user_roles.contains(&edge.role)
            && edge.permlevel <= target_permlevel
            && edge.allows(perm)
    })
}

/// Explicit User Permission rule restricting records by link field (e.g. `Company`, `Cost Center`, `Warehouse`, `Customer`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserPermission {
    pub user: String,
    pub allow_doctype: String,
    pub for_value: String,
    pub is_default: bool,
    pub applicable_for_doctype: Option<String>,
}

/// Ad-hoc document sharing rule (`tabDocShare`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocShare {
    pub user: String,
    pub share_doctype: String,
    pub share_name: String,
    pub read: bool,
    pub write: bool,
    pub submit: bool,
    pub share: bool,
}

/// Dynamic Row-Level Security Engine evaluating `User Permission`, `DocShare`, and role constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPermissionEngine {
    pub user_permissions: Vec<UserPermission>,
    pub doc_shares: Vec<DocShare>,
}

impl UserPermissionEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user permission rule.
    pub fn add_user_permission(
        &mut self,
        user: impl Into<String>,
        allow_doctype: impl Into<String>,
        for_value: impl Into<String>,
        is_default: bool,
        applicable_for: Option<String>,
    ) {
        self.user_permissions.push(UserPermission {
            user: user.into(),
            allow_doctype: allow_doctype.into(),
            for_value: for_value.into(),
            is_default,
            applicable_for_doctype: applicable_for,
        });
    }

    /// Share a document with another user.
    #[allow(clippy::too_many_arguments)]
    pub fn share_document(
        &mut self,
        user: impl Into<String>,
        doctype: impl Into<String>,
        name: impl Into<String>,
        read: bool,
        write: bool,
        submit: bool,
        share: bool,
    ) {
        self.doc_shares.push(DocShare {
            user: user.into(),
            share_doctype: doctype.into(),
            share_name: name.into(),
            read,
            write,
            submit,
            share,
        });
    }

    /// Check if user has explicit ad-hoc document share permission.
    #[must_use]
    pub fn has_doc_share(&self, user: &str, doctype: &str, name: &str, action: Permission) -> bool {
        self.doc_shares.iter().any(|s| {
            s.user == user
                && s.share_doctype == doctype
                && s.share_name == name
                && match action {
                    Permission::Read | Permission::Report | Permission::Export => s.read,
                    Permission::Write | Permission::Execute => s.write,
                    Permission::Submit => s.submit,
                    _ => false,
                }
        })
    }

    /// Compiles SQL/SurrealDB WHERE clause filters for a given user and target DocType.
    #[must_use]
    pub fn compile_user_permission_filters(
        &self,
        user: &str,
        target_doctype: &str,
    ) -> Vec<(String, Vec<String>)> {
        let mut grouped: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for perm in &self.user_permissions {
            if perm.user == user {
                if let Some(app) = &perm.applicable_for_doctype
                    && app != target_doctype
                {
                    continue;
                }
                // Group by allow_doctype (e.g. "Company", "Cost Center")
                grouped
                    .entry(perm.allow_doctype.clone())
                    .or_default()
                    .push(perm.for_value.clone());
            }
        }

        grouped.into_iter().collect()
    }

    /// Builds a SurrealDB/SQL filter string enforcing row-level constraints.
    #[must_use]
    pub fn build_where_clause(
        &self,
        user: &str,
        target_doctype: &str,
        is_system_manager: bool,
    ) -> String {
        if is_system_manager || user == "Administrator" {
            return String::new();
        }

        let filters = self.compile_user_permission_filters(user, target_doctype);
        if filters.is_empty() {
            return String::new();
        }

        let mut parts = Vec::new();
        for (field, values) in filters {
            let field_col = field.to_lowercase().replace(' ', "_");
            let vals = values
                .iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("({field_col} IN [{vals}] OR {field_col} IS NULL)"));
        }

        parts.join(" AND ")
    }
}

/// Generates SurrealDB Row-Level Security (RLS) predicate for a table.
#[must_use]
pub fn compile_rls_policy(
    doctype: &str,
    roles_with_read: &[String],
    roles_with_write: &[String],
) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_permission_registry_defaults() {
        let reg = DynamicRolePermissionRegistry::with_defaults();
        assert!(reg.roles.iter().any(|r| r.name == ROLE_WORKER_USER));
        assert!(reg.roles.iter().any(|r| r.name == ROLE_ACCOUNTANT_USER));
        assert!(reg.roles.iter().any(|r| r.name == ROLE_MARKETING_ADMIN));
        assert!(reg.roles.iter().any(|r| r.name == ROLE_CONTENT_CREATOR));
        assert!(reg.roles.iter().any(|r| r.name == ROLE_WEBSITE_UPDATER));
        assert!(reg.roles.iter().any(|r| r.name == ROLE_WAREHOUSE_MANAGER));

        // Test Worker User permissions
        let worker_roles = vec![ROLE_WORKER_USER.to_string()];
        let worker_job_card = reg.effective_permissions(&worker_roles, "Job Card");
        assert!(worker_job_card.can_read);
        assert!(worker_job_card.can_write);
        assert!(!worker_job_card.can_create);
        assert!(!worker_job_card.can_delete);

        // Worker User cannot access General Ledger
        let worker_gl = reg.effective_permissions(&worker_roles, "GL Entry");
        assert!(!worker_gl.can_read);

        // Test Accountant User permissions
        let acct_roles = vec![ROLE_ACCOUNTANT_USER.to_string()];
        let acct_inv = reg.effective_permissions(&acct_roles, "Sales Invoice");
        assert!(acct_inv.can_read);
        assert!(acct_inv.can_write);
        assert!(acct_inv.can_create);
        assert!(acct_inv.can_submit);
        assert!(acct_inv.can_cancel);

        // Test System Manager bypass
        let admin_roles = vec![ROLE_ADMINISTRATOR.to_string()];
        let admin_perm = reg.effective_permissions(&admin_roles, "AnyDocType");
        assert!(admin_perm.is_admin_bypass);
        assert!(admin_perm.can_delete);
    }

    #[test]
    fn test_user_record_role_management() {
        let mut user = UserRecord::new("usr_test", "Test User", "test@example.com");
        assert!(user.enabled);
        assert!(!user.has_role(ROLE_WAREHOUSE_MANAGER));

        user.add_role(ROLE_WAREHOUSE_MANAGER);
        assert!(user.has_role(ROLE_WAREHOUSE_MANAGER));

        user.remove_role(ROLE_WAREHOUSE_MANAGER);
        assert!(!user.has_role(ROLE_WAREHOUSE_MANAGER));
    }

    #[test]
    fn test_user_permission_engine_where_clause() {
        let mut engine = UserPermissionEngine::new();
        engine.add_user_permission("alice@example.com", "Company", "Acme Inc", true, None);
        engine.add_user_permission(
            "alice@example.com",
            "Cost Center",
            "Sales - AI",
            false,
            Some("Sales Invoice".into()),
        );

        let where_clause = engine.build_where_clause("alice@example.com", "Sales Invoice", false);
        assert!(where_clause.contains("company IN [\"Acme Inc\"]"));
        assert!(where_clause.contains("cost_center IN [\"Sales - AI\"]"));

        // Admin bypass returns empty clause
        assert_eq!(
            engine.build_where_clause("alice@example.com", "Sales Invoice", true),
            ""
        );
        assert_eq!(
            engine.build_where_clause("Administrator", "Sales Invoice", false),
            ""
        );
    }

    #[test]
    fn test_doc_share_evaluation() {
        let mut engine = UserPermissionEngine::new();
        engine.share_document(
            "bob@example.com",
            "Sales Order",
            "SO-2026-0001",
            true,
            false,
            false,
            false,
        );

        assert!(engine.has_doc_share(
            "bob@example.com",
            "Sales Order",
            "SO-2026-0001",
            Permission::Read
        ));
        assert!(!engine.has_doc_share(
            "bob@example.com",
            "Sales Order",
            "SO-2026-0001",
            Permission::Write
        ));
        assert!(!engine.has_doc_share(
            "bob@example.com",
            "Sales Order",
            "SO-2026-0002",
            Permission::Read
        ));
    }

    #[test]
    fn test_check_permission_doctype_scoped() {
        let reg = DynamicRolePermissionRegistry::with_defaults();
        let accountant = vec![ROLE_ACCOUNTANT_USER.to_string()];
        let worker = vec![ROLE_WORKER_USER.to_string()];
        let admin = vec![ROLE_ADMINISTRATOR.to_string()];

        // 1. Accountant can read Sales Invoice
        assert!(check_permission(
            &accountant,
            &reg.permission_edges,
            "Sales Invoice",
            Permission::Read,
            0
        ));

        // 2. Accountant cannot create Timesheet (only worker has create)
        assert!(!check_permission(
            &accountant,
            &reg.permission_edges,
            "Timesheet",
            Permission::Create,
            0
        ));

        // 3. Worker can read Job Card, but NOT GL Entry
        assert!(check_permission(
            &worker,
            &reg.permission_edges,
            "Job Card",
            Permission::Read,
            0
        ));
        assert!(!check_permission(
            &worker,
            &reg.permission_edges,
            "GL Entry",
            Permission::Read,
            0
        ));

        // 4. Doctype scoping prevents cross-doctype permission leakage
        assert!(!check_permission(
            &worker,
            &reg.permission_edges,
            "Sales Invoice",
            Permission::Read,
            0
        ));

        // 5. Admin bypasses all checks on any doctype
        assert!(check_permission(
            &admin,
            &[],
            "AnyDocType",
            Permission::Delete,
            0
        ));
    }
}
