//! Permission Engine (`frappe-framework::permission`).
//!
//! Evaluates Frappe v16 role-based access control (RBAC), field-level permlevels,
//! user permissions (e.g. company restrictions), document ownership (`if_owner`),
//! and dynamic document sharing (`DocShare`).

use crate::lifecycle::Document;
use frappe_meta::{DocPermSchema, DocTypeSchema};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Operation types subject to permission checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermType {
    Read,
    Write,
    Create,
    Delete,
    Submit,
    Cancel,
    Amend,
    Report,
    Export,
    Import,
    Share,
    Print,
    Email,
}

/// Runtime security context of the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserContext {
    /// Unique user identifier (email or username, e.g. "admin@example.com").
    pub user_id: String,
    /// Roles assigned to the user (e.g. ["Accounts User", "Sales Manager"]).
    pub roles: Vec<String>,
    /// Whether the user possesses the System Manager role or is Administrator.
    pub is_system_manager: bool,
    /// Restrictive user permissions (DocType -> Set of permitted link names).
    /// For example: "Company" -> ["Wind Power LLC"].
    pub user_permissions: HashMap<String, HashSet<String>>,
}

impl UserContext {
    /// Creates an Administrator / System Manager superuser context.
    #[must_use]
    pub fn administrator() -> Self {
        Self {
            user_id: "Administrator".to_string(),
            roles: vec!["Administrator".to_string(), "System Manager".to_string()],
            is_system_manager: true,
            user_permissions: HashMap::new(),
        }
    }

    /// Creates a standard user context with specific roles.
    #[must_use]
    pub fn new(user_id: impl Into<String>, roles: Vec<String>) -> Self {
        let is_system_manager = roles
            .iter()
            .any(|r| r == "System Manager" || r == "Administrator");
        Self {
            user_id: user_id.into(),
            roles,
            is_system_manager,
            user_permissions: HashMap::new(),
        }
    }

    /// Adds a user permission constraint.
    pub fn add_user_permission(
        &mut self,
        doctype: impl Into<String>,
        allowed_value: impl Into<String>,
    ) {
        self.user_permissions
            .entry(doctype.into())
            .or_default()
            .insert(allowed_value.into());
    }
}

/// Explicit document share record allowing ad-hoc delegation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocShare {
    /// Target user receiving access.
    pub user: String,
    /// Can read document.
    pub read: bool,
    /// Can modify document.
    pub write: bool,
    /// Can re-share document.
    pub share: bool,
}

/// Core RBAC and Field Permlevel Authorization Engine.
pub struct PermissionEngine;

impl PermissionEngine {
    /// Checks if a user has permission to perform `ptype` on a `Document`.
    #[must_use]
    pub fn has_permission(
        doc: &Document,
        ptype: PermType,
        user: &UserContext,
        schema: &DocTypeSchema,
        shares: &[DocShare],
    ) -> bool {
        // 1. Superuser / System Manager bypass
        if user.is_system_manager || user.user_id == "Administrator" {
            return true;
        }

        // 2. Check User Permissions (e.g. restriction by Company)
        if !Self::check_user_permissions(doc, user) {
            return false;
        }

        // 3. Check DocShare explicit delegations
        for share in shares {
            if share.user == user.user_id {
                match ptype {
                    PermType::Read | PermType::Print | PermType::Email | PermType::Report
                        if share.read =>
                    {
                        return true;
                    }
                    PermType::Write if share.write => {
                        return true;
                    }
                    PermType::Share if share.share => {
                        return true;
                    }
                    _ => {}
                }
            }
        }

        // 4. Check DocType Schema permissions (Permlevel 0)
        let doc_owner = doc.data.get("owner").and_then(|v| v.as_str()).unwrap_or("");
        let is_owner = !doc_owner.is_empty() && doc_owner == user.user_id;

        for perm in &schema.permissions {
            if perm.permlevel != 0 {
                continue;
            }

            let role_matches = perm.role == "All" || user.roles.iter().any(|r| r == &perm.role);
            if !role_matches {
                continue;
            }

            // In Frappe, if perm applies to owner, verify user is owner
            if perm.role == "Owner" && !is_owner {
                continue;
            }

            if Self::perm_grants(perm, ptype) {
                return true;
            }
        }

        false
    }

    /// Evaluates if an individual `DocPermSchema` grants the requested operation.
    #[must_use]
    fn perm_grants(perm: &DocPermSchema, ptype: PermType) -> bool {
        match ptype {
            PermType::Read => perm.read,
            PermType::Write => perm.write,
            PermType::Create => perm.create,
            PermType::Delete => perm.delete,
            PermType::Submit => perm.submit,
            PermType::Cancel => perm.cancel,
            PermType::Amend => perm.amend,
            PermType::Report => perm.report,
            PermType::Export => perm.export,
            PermType::Import => perm.import,
            PermType::Share => perm.write,
            PermType::Print => perm.read,
            PermType::Email => perm.read,
        }
    }

    /// Validates user permission link constraints (e.g. `company` field).
    #[must_use]
    fn check_user_permissions(doc: &Document, user: &UserContext) -> bool {
        if user.user_permissions.is_empty() {
            return true;
        }

        let Some(obj) = doc.data.as_object() else {
            return true;
        };

        for (perm_doctype, allowed_set) in &user.user_permissions {
            let fieldname = perm_doctype.to_lowercase().replace(' ', "_");
            if let Some(val) = obj.get(&fieldname).and_then(|v| v.as_str())
                && !allowed_set.contains(val)
            {
                return false;
            }
        }

        true
    }

    /// Filters document fields based on field-level permlevels (0..9).
    /// If user does not have read (or write) at the field's permlevel, the field is stripped.
    #[must_use]
    pub fn filter_document_fields(
        doc_data: &serde_json::Value,
        user: &UserContext,
        schema: &DocTypeSchema,
        is_write: bool,
    ) -> serde_json::Value {
        if user.is_system_manager || user.user_id == "Administrator" {
            return doc_data.clone();
        }

        let Some(obj) = doc_data.as_object() else {
            return doc_data.clone();
        };

        // Compute set of permitted permlevels for this user
        let mut allowed_permlevels = HashSet::new();
        allowed_permlevels.insert(0); // Permlevel 0 is governed at document level

        for perm in &schema.permissions {
            let role_matches = perm.role == "All" || user.roles.iter().any(|r| r == &perm.role);
            if !role_matches {
                continue;
            }

            let granted = if is_write { perm.write } else { perm.read };
            if granted {
                allowed_permlevels.insert(perm.permlevel);
            }
        }

        let mut filtered = serde_json::Map::new();
        for field in &schema.fields {
            if allowed_permlevels.contains(&field.permlevel)
                && let Some(val) = obj.get(&field.fieldname)
            {
                filtered.insert(field.fieldname.clone(), val.clone());
            }
        }

        // Always preserve system metadata fields
        for sys_field in [
            "name",
            "doctype",
            "docstatus",
            "owner",
            "modified",
            "creation",
        ] {
            if let Some(val) = obj.get(sys_field) {
                filtered.insert(sys_field.to_string(), val.clone());
            }
        }

        serde_json::Value::Object(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frappe_meta::{DocFieldSchema, FieldType};

    fn make_secured_schema() -> DocTypeSchema {
        DocTypeSchema {
            name: "Salary Slip".into(),
            module: "Payroll".into(),
            is_submittable: true,
            is_single: false,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            title_field: None,
            search_fields: None,
            image_field: None,
            fields: vec![
                DocFieldSchema {
                    fieldname: "employee".into(),
                    label: "Employee".into(),
                    fieldtype: FieldType::Data,
                    reqd: true,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 0,
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
                DocFieldSchema {
                    fieldname: "net_pay".into(),
                    label: "Net Pay".into(),
                    fieldtype: FieldType::Currency,
                    reqd: true,
                    unique: false,
                    read_only: false,
                    hidden: false,
                    in_list_view: true,
                    mask: false,
                    options: None,
                    default_value: None,
                    permlevel: 1, // Restricted field
                    depends_on: None,
                    mandatory_depends_on: None,
                    read_only_depends_on: None,
                    fetch_from: None,
                    precision: None,
                },
            ],
            permissions: vec![
                DocPermSchema {
                    role: "Employee".into(),
                    read: true,
                    write: false,
                    create: false,
                    delete: false,
                    submit: false,
                    cancel: false,
                    amend: false,
                    report: false,
                    export: false,
                    import: false,
                    permlevel: 0,
                    accounting_period_exempt: false,
                },
                DocPermSchema {
                    role: "HR Manager".into(),
                    read: true,
                    write: true,
                    create: true,
                    delete: true,
                    submit: true,
                    cancel: true,
                    amend: true,
                    report: true,
                    export: true,
                    import: true,
                    permlevel: 0,
                    accounting_period_exempt: false,
                },
                DocPermSchema {
                    role: "HR Manager".into(),
                    read: true,
                    write: true,
                    create: false,
                    delete: false,
                    submit: false,
                    cancel: false,
                    amend: false,
                    report: false,
                    export: false,
                    import: false,
                    permlevel: 1, // HR Manager can read/write permlevel 1
                    accounting_period_exempt: false,
                },
            ],
        }
    }

    #[test]
    fn test_role_and_permlevel_evaluation() {
        let schema = make_secured_schema();
        let doc = Document::new(
            "Salary Slip",
            serde_json::json!({
                "name": "SAL-001",
                "employee": "EMP-001",
                "net_pay": 5000,
                "company": "Tech Corp"
            }),
        );

        let employee_user = UserContext::new("emp1@test.com", vec!["Employee".into()]);
        let hr_user = UserContext::new("hr@test.com", vec!["HR Manager".into()]);
        let guest_user = UserContext::new("guest@test.com", vec!["Guest".into()]);

        // Employee can read but not write or submit
        assert!(PermissionEngine::has_permission(
            &doc,
            PermType::Read,
            &employee_user,
            &schema,
            &[]
        ));
        assert!(!PermissionEngine::has_permission(
            &doc,
            PermType::Write,
            &employee_user,
            &schema,
            &[]
        ));
        assert!(!PermissionEngine::has_permission(
            &doc,
            PermType::Submit,
            &employee_user,
            &schema,
            &[]
        ));

        // HR Manager has full access
        assert!(PermissionEngine::has_permission(
            &doc,
            PermType::Submit,
            &hr_user,
            &schema,
            &[]
        ));

        // Guest has no access
        assert!(!PermissionEngine::has_permission(
            &doc,
            PermType::Read,
            &guest_user,
            &schema,
            &[]
        ));

        // Permlevel 1 field filtering
        let employee_view =
            PermissionEngine::filter_document_fields(&doc.data, &employee_user, &schema, false);
        assert_eq!(employee_view["employee"], "EMP-001");
        assert!(employee_view.get("net_pay").is_none()); // Strip restricted field!

        let hr_view = PermissionEngine::filter_document_fields(&doc.data, &hr_user, &schema, false);
        assert_eq!(hr_view["employee"], "EMP-001");
        assert_eq!(hr_view["net_pay"], 5000); // HR Manager sees net_pay
    }

    #[test]
    fn test_user_permission_restriction() {
        let schema = make_secured_schema();
        let doc = Document::new(
            "Salary Slip",
            serde_json::json!({
                "employee": "EMP-001",
                "company": "Other Corp"
            }),
        );

        let mut user = UserContext::new("emp@test.com", vec!["Employee".into()]);
        user.add_user_permission("Company", "Tech Corp");

        // Allowed only for Tech Corp, doc is Other Corp -> Denied!
        assert!(!PermissionEngine::has_permission(
            &doc,
            PermType::Read,
            &user,
            &schema,
            &[]
        ));
    }
}
