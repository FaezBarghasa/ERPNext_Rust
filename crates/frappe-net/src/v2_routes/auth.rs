//! Authentication, MFA, and Session Management routes.

use super::*;
use actix_web::web;

/// Configures all Authentication and Session management endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v2/method/login", web::post().to(login_handler))
        .route("/api/v2/method/logout", web::post().to(logout_handler))
        .route(
            "/api/v2/auth/refresh",
            web::post().to(auth_refresh_token_handler),
        )
        .route(
            "/api/v2/auth/mfa/enroll",
            web::post().to(auth_mfa_enroll_handler),
        )
        .route(
            "/api/v2/auth/mfa/activate",
            web::post().to(auth_mfa_activate_handler),
        )
        .route(
            "/api/v2/auth/mfa/disable",
            web::post().to(auth_mfa_disable_handler),
        )
        .route(
            "/api/v2/auth/password/forgot",
            web::post().to(auth_forgot_password_handler),
        )
        .route(
            "/api/v2/auth/password/reset",
            web::post().to(auth_reset_password_handler),
        );
}
