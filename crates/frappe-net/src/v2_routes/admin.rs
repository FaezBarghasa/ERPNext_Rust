//! RBAC Dynamic Roles, IP Filtering Rules, Audit Logs, and Account Lockout routes.

use super::*;
use actix_web::web;

/// Configures all Administrative, Dynamic Role, IP Filter, and Audit endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v2/admin/status", web::get().to(admin_status_handler))
        .route(
            "/api/v2/admin/action/{action_id}",
            web::post().to(admin_action_handler),
        )
        .route("/api/v2/admin/query", web::post().to(admin_query_handler))
        .route(
            "/api/v2/admin/users",
            web::get().to(admin_list_users_handler),
        )
        .route(
            "/api/v2/admin/users",
            web::post().to(admin_create_user_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}/roles",
            web::put().to(admin_update_user_roles_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}",
            web::delete().to(admin_delete_user_handler),
        )
        .route(
            "/api/v2/admin/users/{user_id}/effective-permissions",
            web::get().to(admin_get_user_effective_permissions_handler),
        )
        .route(
            "/api/v2/admin/roles",
            web::get().to(admin_list_roles_handler),
        )
        .route(
            "/api/v2/admin/roles",
            web::post().to(admin_create_role_handler),
        )
        .route(
            "/api/v2/admin/permissions",
            web::get().to(admin_get_permissions_handler),
        )
        .route(
            "/api/v2/admin/permissions",
            web::post().to(admin_update_permission_handler),
        )
        .route(
            "/api/v2/admin/audit-logs",
            web::get().to(admin_get_audit_logs_handler),
        )
        .route(
            "/api/v2/admin/ip-rules",
            web::get().to(admin_list_ip_rules_handler),
        )
        .route(
            "/api/v2/admin/ip-rules",
            web::post().to(admin_create_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/ip-rules/{rule_id}",
            web::delete().to(admin_delete_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/lockouts",
            web::get().to(admin_list_lockouts_handler),
        )
        .route(
            "/api/v2/admin/lockouts/{target}/unlock",
            web::post().to(admin_unlock_target_handler),
        );
}
