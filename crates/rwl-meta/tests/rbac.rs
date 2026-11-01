use rwl_meta::rbac::{
    HasPermissionEdge, Permission, ROLE_ADMINISTRATOR, ROLE_SYSTEM_MANAGER, check_permission,
};

#[test]
fn test_admin_and_system_manager_bypass() {
    let edges = vec![]; // No explicit permissions granted

    let admin_roles = vec![ROLE_ADMINISTRATOR.to_string()];
    let sysman_roles = vec![ROLE_SYSTEM_MANAGER.to_string()];

    // Both should bypass and have access to all DocTypes and permissions
    assert!(check_permission(
        &admin_roles,
        &edges,
        "Salary Slip",
        Permission::Read,
        0
    ));
    assert!(check_permission(
        &sysman_roles,
        &edges,
        "Journal Entry",
        Permission::Delete,
        0
    ));
}

#[test]
fn test_role_permission_grant_and_deny() {
    let edges = vec![
        HasPermissionEdge {
            role: "Accounts User".into(),
            doctype: "Sales Invoice".into(),
            p_read: true,
            p_write: true,
            p_create: true,
            p_delete: false,
            p_submit: false,
            p_cancel: false,
            p_amend: false,
            permlevel: 0,
        },
        HasPermissionEdge {
            role: "Accounts Manager".into(),
            doctype: "Sales Invoice".into(),
            p_read: true,
            p_write: true,
            p_create: true,
            p_delete: true,
            p_submit: true,
            p_cancel: true,
            p_amend: true,
            permlevel: 0,
        },
    ];

    let user_roles = vec!["Accounts User".to_string()];

    // Allowed
    assert!(check_permission(
        &user_roles,
        &edges,
        "Sales Invoice",
        Permission::Read,
        0
    ));
    assert!(check_permission(
        &user_roles,
        &edges,
        "Sales Invoice",
        Permission::Write,
        0
    ));

    // Denied for Accounts User
    assert!(!check_permission(
        &user_roles,
        &edges,
        "Sales Invoice",
        Permission::Delete,
        0
    ));
    assert!(!check_permission(
        &user_roles,
        &edges,
        "Sales Invoice",
        Permission::Submit,
        0
    ));

    // Denied: unauthorized DocType
    assert!(!check_permission(
        &user_roles,
        &edges,
        "Salary Slip",
        Permission::Read,
        0
    ));

    // Allowed for Accounts Manager
    let mgr_roles = vec!["Accounts Manager".to_string()];
    assert!(check_permission(
        &mgr_roles,
        &edges,
        "Sales Invoice",
        Permission::Submit,
        0
    ));
    assert!(check_permission(
        &mgr_roles,
        &edges,
        "Sales Invoice",
        Permission::Delete,
        0
    ));
}

#[test]
fn test_permlevel_security_boundary() {
    let edges = vec![
        HasPermissionEdge {
            role: "Employee".into(),
            doctype: "Salary Slip".into(),
            p_read: true,
            p_write: false,
            p_create: false,
            p_delete: false,
            p_submit: false,
            p_cancel: false,
            p_amend: false,
            permlevel: 0, // General record level
        },
        HasPermissionEdge {
            role: "HR Manager".into(),
            doctype: "Salary Slip".into(),
            p_read: true,
            p_write: true,
            p_create: true,
            p_delete: true,
            p_submit: true,
            p_cancel: true,
            p_amend: true,
            permlevel: 1, // Confidential fields level
        },
    ];

    let employee_roles = vec!["Employee".to_string()];

    // Employee has permlevel 0 access
    assert!(check_permission(
        &employee_roles,
        &edges,
        "Salary Slip",
        Permission::Read,
        0
    ));

    // HR Manager with permlevel 1 edge has access to both permlevel 0 and 1
    let hr_roles = vec!["HR Manager".to_string()];
    assert!(check_permission(
        &hr_roles,
        &edges,
        "Salary Slip",
        Permission::Read,
        1
    ));
}

