//! Generic REST Resource Handlers with RBAC Guarding (`frappe-net::routes`).

use crate::middleware::auth::SecurityContext;
use crate::tenant::{ConnectionPoolManager, TenantId};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use frappe_framework::{Document, DocumentController};
use frappe_meta::{DocTypeSchema, Permission, check_permission};
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

static SAFE_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Regex compile"));

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<String>,
}

/// Generic handler for listing documents: `GET /api/v1/resource/{doctype}`
pub async fn list_resource(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<ListQuery>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let doctype = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>() {
        if !check_permission(&ctx.claims.roles, &[], Permission::Read, 0) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission Denied: insufficient read privileges"
            }));
        }
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_ID_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let sql = format!("SELECT * FROM {table} LIMIT {limit} START {offset};");
    match client.query(&sql).await {
        Ok(mut res) => {
            let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(records)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Generic handler for getting a document by ID: `GET /api/v1/resource/{doctype}/{id}`
pub async fn get_resource(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, id) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>() {
        if !check_permission(&ctx.claims.roles, &[], Permission::Read, 0) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission Denied: insufficient read privileges"
            }));
        }
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_ID_REGEX.is_match(&table) || !SAFE_ID_REGEX.is_match(&id) {
        return HttpResponse::BadRequest().body("Invalid identifier");
    }

    let sql = format!("SELECT * FROM {table}:{id};");
    match client.query(&sql).await {
        Ok(mut res) => {
            let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            match record {
                Some(r) => HttpResponse::Ok().json(r),
                None => HttpResponse::NotFound().body("Record not found"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Generic handler for creating a document: `POST /api/v1/resource/{doctype}`
pub async fn create_resource(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let doctype = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>() {
        if !check_permission(&ctx.claims.roles, &[], Permission::Create, 0) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission Denied: insufficient create privileges"
            }));
        }
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let schema = DocTypeSchema {
        name: doctype.clone(),
        module: "Core".to_string(),
        is_single: false,
        is_submittable: false,
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
        fields: vec![],
        permissions: vec![],
    };

    let mut doc = Document::new(&doctype, body.into_inner());
    let controller = DocumentController::new();

    if let Err(e) = controller.insert(&mut doc, &schema, None, 2026, 1) {
        return HttpResponse::BadRequest().body(e.to_string());
    }

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_ID_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let sql = format!("CREATE {table} CONTENT $doc;");
    match client.query(&sql).bind(("doc", doc.data)).await {
        Ok(mut res) => {
            let created: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            HttpResponse::Created().json(created)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Generic handler for deleting a document: `DELETE /api/v1/resource/{doctype}/{id}`
pub async fn delete_resource(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, id) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>() {
        if !check_permission(&ctx.claims.roles, &[], Permission::Delete, 0) {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Permission Denied: insufficient delete privileges"
            }));
        }
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_ID_REGEX.is_match(&table) || !SAFE_ID_REGEX.is_match(&id) {
        return HttpResponse::BadRequest().body("Invalid identifier");
    }

    let sql = format!("DELETE {table}:{id};");
    match client.query(&sql).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
