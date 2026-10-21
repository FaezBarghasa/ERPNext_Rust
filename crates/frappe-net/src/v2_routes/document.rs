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
    .route("/files/image/{hash}", web::get().to(download_image_handler))
    // Pure Dynamic Meta-Model Engine REST API routes
    .route("/api/v2/meta/doctype", web::get().to(v2_list_meta_doctypes))
    .route("/api/v2/meta/doctype", web::post().to(v2_create_custom_doctype))
    .route("/api/v2/meta/doctype/{name}", web::get().to(v2_get_meta_doctype))
    .route("/api/v2/meta/doctype/{name}", web::put().to(v2_update_custom_doctype))
    .route("/api/v2/meta/doctype/{name}", web::delete().to(v2_delete_custom_doctype))
    .route("/api/v2/meta/doctype/{name}/custom_field", web::post().to(v2_add_custom_field))
    .route("/api/v2/meta/snapshot", web::get().to(v2_export_meta_snapshot))
    .route("/api/v2/meta/snapshot", web::post().to(v2_import_meta_snapshot));
}

/// Handler for listing all DocTypes: `GET /api/v2/meta/doctype`
pub async fn v2_list_meta_doctypes() -> impl Responder {
    let registry = frappe_meta::global_schema_registry();
    let doctypes = registry.list_doctypes();
    HttpResponse::Ok().json(serde_json::json!({
        "data": doctypes
    }))
}

/// Handler for retrieving a single DocType schema: `GET /api/v2/meta/doctype/{name}`
pub async fn v2_get_meta_doctype(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let registry = frappe_meta::global_schema_registry();
    match registry.get_schema(&name) {
        Some(schema) => HttpResponse::Ok().json(serde_json::json!({
            "data": schema.as_ref()
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("DocType '{name}' not found")
        })),
    }
}

/// Handler for creating a custom DocType: `POST /api/v2/meta/doctype`
pub async fn v2_create_custom_doctype(
    req: HttpRequest,
    payload: web::Json<frappe_meta::CustomDocType>,
    pool_mgr: Option<web::Data<ConnectionPoolManager>>,
) -> impl Responder {
    let custom = payload.into_inner();
    let registry = frappe_meta::global_schema_registry();
    let tenant_id_opt = req.extensions().get::<TenantId>().cloned();
    match registry.create_custom_doctype(custom) {
        Ok(schema) => {
            let ddl = registry.compile_surrealql_ddl(&schema.name).unwrap_or_default();
            if let (Some(pool), Some(tenant_id)) = (pool_mgr, tenant_id_opt)
                && let Ok(client) = pool.get_or_initialize_client(&tenant_id).await
            {
                for stmt in &ddl {
                    let _ = client.query(stmt).await;
                }
            }
            HttpResponse::Created().json(serde_json::json!({
                "data": schema.as_ref(),
                "ddl": ddl
            }))
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Handler for updating an existing custom DocType: `PUT /api/v2/meta/doctype/{name}`
pub async fn v2_update_custom_doctype(
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<frappe_meta::CustomDocType>,
    pool_mgr: Option<web::Data<ConnectionPoolManager>>,
) -> impl Responder {
    let name = path.into_inner();
    let custom = payload.into_inner();
    let registry = frappe_meta::global_schema_registry();
    let tenant_id_opt = req.extensions().get::<TenantId>().cloned();
    match registry.update_custom_doctype(&name, custom) {
        Ok(schema) => {
            let ddl = registry.compile_surrealql_ddl(&schema.name).unwrap_or_default();
            if let (Some(pool), Some(tenant_id)) = (pool_mgr, tenant_id_opt)
                && let Ok(client) = pool.get_or_initialize_client(&tenant_id).await
            {
                for stmt in &ddl {
                    let _ = client.query(stmt).await;
                }
            }
            HttpResponse::Ok().json(serde_json::json!({
                "data": schema.as_ref(),
                "ddl": ddl
            }))
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Handler for deleting a custom DocType: `DELETE /api/v2/meta/doctype/{name}`
pub async fn v2_delete_custom_doctype(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let registry = frappe_meta::global_schema_registry();
    match registry.delete_custom_doctype(&name) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("DocType '{name}' deleted successfully")
        })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("DocType '{name}' was not found in registry")
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Handler for adding a custom field: `POST /api/v2/meta/doctype/{name}/custom_field`
pub async fn v2_add_custom_field(
    path: web::Path<String>,
    payload: web::Json<frappe_meta::CustomField>,
) -> impl Responder {
    let name = path.into_inner();
    let mut custom_field = payload.into_inner();
    if custom_field.dt.is_empty() {
        custom_field.dt = name.clone().into();
    }
    let registry = frappe_meta::global_schema_registry();
    match registry.add_custom_field(custom_field) {
        Ok(()) => {
            let updated_schema = registry.get_schema(&name);
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Custom field added to DocType '{name}'"),
                "data": updated_schema.as_deref()
            }))
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Handler for exporting schema snapshot: `GET /api/v2/meta/snapshot`
pub async fn v2_export_meta_snapshot() -> impl Responder {
    let registry = frappe_meta::global_schema_registry();
    let snapshot = registry.export_snapshot();
    HttpResponse::Ok().json(snapshot)
}

/// Handler for importing schema snapshot: `POST /api/v2/meta/snapshot`
pub async fn v2_import_meta_snapshot(payload: web::Json<serde_json::Value>) -> impl Responder {
    let registry = frappe_meta::global_schema_registry();
    match registry.import_snapshot(&payload.into_inner()) {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully imported {count} schemas"),
            "imported_count": count
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
