//! Dynamic DocType CRUD, Hypermedia, Lifecycle, and CAS Storage routes.

use super::*;
use actix_web::web;

/// Configures all Dynamic DocType Document CRUD and Storage endpoints.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/v2/document/{doctype}",
        web::get().to(v2_list_document),
    )
    .route(
        "/api/v2/document/{doctype}",
        web::post().to(v2_create_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}",
        web::get().to(v2_get_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}",
        web::put().to(v2_update_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}",
        web::delete().to(v2_delete_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}/submit",
        web::post().to(v2_submit_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}/cancel",
        web::post().to(v2_cancel_document),
    )
    .route(
        "/api/v2/document/{doctype}/{docname}/amend",
        web::post().to(v2_amend_document),
    )
    .route(
        "/api/v2/method/upload_file",
        web::post().to(upload_file_handler),
    )
    .route("/files/{hash}", web::get().to(download_file_handler))
    .route("/files/image/{hash}", web::get().to(download_image_handler));
}
