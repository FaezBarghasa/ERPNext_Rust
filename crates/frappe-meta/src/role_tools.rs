//! Role Permissions, Masking & Replication Tooling (`frappe-meta::role_tools`).
//!
//! Provides role replication, export/import, accounting period role exemption,
//! and granular role-based field masking evaluation.

use crate::rbac::Role;
use crate::schema::{DocFieldSchema, DocPermSchema};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Serialized role bundle for replication across sites.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleReplicationBundle {
    pub source_site: String,
    pub roles: Vec<Role>,
    pub doctype_permissions: Vec<(String, Vec<DocPermSchema>)>,
}

impl RoleReplicationBundle {
    #[must_use]
    pub fn new(source_site: impl Into<String>) -> Self {
        Self {
            source_site: source_site.into(),
            roles: Vec::new(),
            doctype_permissions: Vec::new(),
        }
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.push(role);
    }

    pub fn add_doctype_permission(&mut self, doctype: impl Into<String>, perms: Vec<DocPermSchema>) {
        self.doctype_permissions.push((doctype.into(), perms));
    }
}

/// Evaluator for Role-Based Field Masking and Accounting Period Exemption.
pub struct RoleEvaluator;

impl RoleEvaluator {
    /// Evaluates whether a field value should be displayed as masked (e.g. `******`)
    /// based on the field's `mask` attribute and the user's active roles.
    #[must_use]
    pub fn should_mask_field(
        field: &DocFieldSchema,
        user_roles: &HashSet<CompactString>,
        unmask_roles: &[&str],
    ) -> bool {
        if !field.mask {
            return false;
        }

        // If user has any role authorized to unmask, return false (do not mask)
        for role in unmask_roles {
            if user_roles.contains(*role) {
                return false;
            }
        }

        // Default: mask the field
        true
    }

    /// Evaluates whether the given user roles are exempt from closed accounting period restrictions.
    #[must_use]
    pub fn is_accounting_period_exempt(
        user_roles: &HashSet<CompactString>,
        permissions: &[DocPermSchema],
    ) -> bool {
        for perm in permissions {
            if perm.accounting_period_exempt && user_roles.contains(perm.role.as_str()) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::FieldType;

    #[test]
    fn test_role_masking_and_accounting_exemption() {
        let field = DocFieldSchema {
            fieldname: "iban".into(),
            fieldtype: FieldType::Data,
            label: "Bank Account IBAN".into(),
            reqd: false,
            unique: false,
            read_only: false,
            hidden: false,
            in_list_view: false,
            mask: true,
            options: None,
            default_value: None,
            permlevel: 0,
        };

        let mut user_roles = HashSet::new();
        user_roles.insert(CompactString::new("Accounts User"));

        // Regular accounts user should see masked field
        assert!(RoleEvaluator::should_mask_field(&field, &user_roles, &["Accounts Manager", "System Manager"]));

        // Manager should see unmasked field
        user_roles.insert(CompactString::new("Accounts Manager"));
        assert!(!RoleEvaluator::should_mask_field(&field, &user_roles, &["Accounts Manager", "System Manager"]));

        let perms = vec![DocPermSchema {
            role: "Accounts Manager".into(),
            read: true,
            write: true,
            create: true,
            delete: false,
            submit: true,
            cancel: true,
            amend: true,
            report: true,
            export: true,
            import: false,
            permlevel: 0,
            accounting_period_exempt: true,
        }];

        assert!(RoleEvaluator::is_accounting_period_exempt(&user_roles, &perms));
    }
}
