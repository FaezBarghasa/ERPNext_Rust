//! API V2 Hypermedia RESTful Architecture & System Endpoints (`frappe-net::v2_routes`).
//!
//! Provides:
//! - `/api/v2/document/:doctype` (GET list with filters/sort/page, POST create document)
//! - `/api/v2/document/:doctype/:name` (GET single, PUT update, DELETE document)
//! - `/api/v2/document/:doctype/:name/submit` (Submit document: docstatus 0 -> 1)
//! - `/api/v2/document/:doctype/:name/cancel` (Cancel document: docstatus 1 -> 2)
//! - `/api/v2/document/:doctype/:name/amend` (Amend cancelled document: creates draft clone)
//! - `/api/v2/method/upload_file` (Content-Addressable Storage upload with SHA-256 deduplication)
//! - `/files/:hash` (Streaming file asset download with caching headers)
//! - System endpoints: `/api/v2/method/login`, `logout`, `ping`.

use crate::middleware::auth::{SecurityContext, get_master_token_secret};
use crate::tenant::{ConnectionPoolManager, TenantId};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use chrono::Utc;
use frappe_meta::auth::{SessionClaims, hash_password, issue_token, verify_password};
use frappe_meta::rbac::{Permission, check_permission};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::LazyLock;

static SAFE_FIELD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_.]*$").expect("Regex compile"));

static SAFE_ORDER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_.]*(\s+(?i:asc|desc))?$").expect("Regex compile")
});

/// Allowed safe operators for SurrealQL filter clauses.
const ALLOWED_OPERATORS: &[&str] = &[
    "=",
    "!=",
    "<",
    "<=",
    ">",
    ">=",
    "CONTAINS",
    "CONTAINSNOT",
    "LIKE",
    "NOT LIKE",
    "IN",
    "NOT IN",
];

#[derive(Debug, Deserialize)]
pub struct V2ListQuery {
    pub fields: Option<String>,
    pub filters: Option<String>,
    pub order_by: Option<String>,
    pub limit_start: Option<usize>,
    pub limit_page_length: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub usr: String,
    pub pwd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFilePayload {
    pub file_name: String,
    pub content_base64: String,
    pub is_private: Option<bool>,
}

/// Sanitizes a string literal by escaping backslashes and double quotes.
#[must_use]
pub fn sanitize_surrealql_string(val: &str) -> String {
    val.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Helper function to convert JSON filter list `[["status", "=", "Open"]]` into a safe SurrealQL WHERE clause.
pub fn compile_filters_to_surrealql(filters_json: &str) -> Option<String> {
    if let Ok(filters) = serde_json::from_str::<Vec<Vec<String>>>(filters_json) {
        if filters.is_empty() {
            return None;
        }
        let clauses: Vec<String> = filters
            .iter()
            .filter_map(|f| {
                if f.len() >= 3 {
                    let field = f[0].trim();
                    let op = f[1].trim().to_uppercase();
                    let val = f[2].trim();

                    // Security: Verify field identifier matches safe regex
                    if !SAFE_FIELD_REGEX.is_match(field) || field.len() > 64 {
                        return None;
                    }

                    // Security: Verify operator is in the strict allowlist
                    if !ALLOWED_OPERATORS.contains(&op.as_str()) {
                        return None;
                    }

                    // Security: Escape string value
                    let sanitized_val = sanitize_surrealql_string(val);
                    Some(format!("{field} {op} \"{sanitized_val}\""))
                } else {
                    None
                }
            })
            .collect();
        if !clauses.is_empty() {
            return Some(clauses.join(" AND "));
        }
    }
    None
}

/// Handler for listing documents: `GET /api/v2/document/{doctype}`
pub async fn v2_list_document(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<V2ListQuery>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let doctype = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Read, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient read privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_FIELD_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let fields_clause = if let Some(ref f) = query.fields {
        if let Ok(field_list) = serde_json::from_str::<Vec<String>>(f) {
            let valid_fields: Vec<String> = field_list
                .into_iter()
                .filter(|field| SAFE_FIELD_REGEX.is_match(field.trim()) && field.len() <= 64)
                .collect();
            if valid_fields.is_empty() {
                "*".to_string()
            } else {
                valid_fields.join(", ")
            }
        } else {
            "*".to_string()
        }
    } else {
        "*".to_string()
    };

    let where_clause = query
        .filters
        .as_deref()
        .and_then(compile_filters_to_surrealql)
        .map(|c| format!("WHERE {c}"))
        .unwrap_or_default();

    let order_clause = if let Some(ref o) = query.order_by {
        if SAFE_ORDER_REGEX.is_match(o.trim()) {
            format!("ORDER BY {}", o.trim())
        } else {
            "ORDER BY creation DESC".to_string()
        }
    } else {
        "ORDER BY creation DESC".to_string()
    };

    let limit = query.limit_page_length.unwrap_or(20);
    let start = query.limit_start.unwrap_or(0);

    let sql = format!(
        "SELECT {fields_clause} FROM {table} {where_clause} {order_clause} LIMIT {limit} START {start};"
    );
    match client.query(&sql).await {
        Ok(mut res) => {
            let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(serde_json::json!({
                "data": records,
                "page_length": limit,
                "start": start,
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for getting single document: `GET /api/v2/document/{doctype}/{name}`
pub async fn v2_get_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Read, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient read privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_FIELD_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let sanitized_name = sanitize_surrealql_string(&name);
    let sql = format!("SELECT * FROM {table} WHERE name = '{sanitized_name}';");
    match client.query(&sql).await {
        Ok(mut res) => {
            let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            match records.into_iter().next() {
                Some(r) => HttpResponse::Ok().json(serde_json::json!({ "doc": r })),
                None => HttpResponse::NotFound().body("Document not found"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for creating a document: `POST /api/v2/document/{doctype}`
pub async fn v2_create_document(
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let doctype = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Create, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient create privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_FIELD_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let mut doc_map = match payload.into_inner() {
        serde_json::Value::Object(m) => m,
        _ => return HttpResponse::BadRequest().body("Payload must be a JSON object"),
    };

    let name = if let Some(serde_json::Value::String(n)) = doc_map.get("name") {
        n.clone()
    } else {
        format!("{}-{}", table, Utc::now().timestamp_millis())
    };

    let now_str = Utc::now().to_rfc3339();
    doc_map.insert("name".into(), serde_json::Value::String(name.clone()));
    doc_map.insert("doctype".into(), serde_json::Value::String(doctype.clone()));
    doc_map.insert("docstatus".into(), serde_json::Value::Number(0.into()));
    doc_map.insert(
        "creation".into(),
        serde_json::Value::String(now_str.clone()),
    );
    doc_map.insert("modified".into(), serde_json::Value::String(now_str));

    let query_sql = format!("CREATE {table} CONTENT $doc;");
    match client
        .query(&query_sql)
        .bind(("doc", serde_json::Value::Object(doc_map.clone())))
        .await
    {
        Ok(mut res) => {
            let created: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            HttpResponse::Created().json(serde_json::json!({
                "doc": created.unwrap_or(serde_json::Value::Object(doc_map))
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for updating a document: `PUT /api/v2/document/{doctype}/{name}`
pub async fn v2_update_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    payload: web::Json<serde_json::Value>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Write, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient write privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_FIELD_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let mut doc_map = match payload.into_inner() {
        serde_json::Value::Object(m) => m,
        _ => return HttpResponse::BadRequest().body("Payload must be a JSON object"),
    };

    doc_map.insert(
        "modified".into(),
        serde_json::Value::String(Utc::now().to_rfc3339()),
    );

    let sanitized_name = sanitize_surrealql_string(&name);
    let query_sql = format!("UPDATE {table} MERGE $doc WHERE name = '{sanitized_name}';");
    match client
        .query(&query_sql)
        .bind(("doc", serde_json::Value::Object(doc_map.clone())))
        .await
    {
        Ok(mut res) => {
            let updated: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            match updated.into_iter().next() {
                Some(u) => HttpResponse::Ok().json(serde_json::json!({ "doc": u })),
                None => HttpResponse::NotFound().body("Document not found for update"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for deleting a document: `DELETE /api/v2/document/{doctype}/{name}`
pub async fn v2_delete_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Delete, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient delete privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    if !SAFE_FIELD_REGEX.is_match(&table) {
        return HttpResponse::BadRequest().body("Invalid doctype identifier");
    }

    let sanitized_name = sanitize_surrealql_string(&name);

    // Guard: Prevent deleting submitted document (docstatus == 1)
    let check_sql = format!("SELECT docstatus FROM {table} WHERE name = '{sanitized_name}';");
    if let Ok(mut res) = client.query(&check_sql).await {
        let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
        if let Some(r) = records.into_iter().next()
            && let Some(docstatus) = r.get("docstatus").and_then(|d| d.as_i64())
            && docstatus == 1
        {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Cannot delete submitted document. Cancel document first."
            }));
        }
    }

    let sql = format!("DELETE {table} WHERE name = '{sanitized_name}';");
    match client.query(&sql).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Document deleted successfully",
            "name": name
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for submitting a document: `POST /api/v2/document/{doctype}/{name}/submit`
pub async fn v2_submit_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Submit, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient submit privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    let sanitized_name = sanitize_surrealql_string(&name);
    let now = Utc::now().to_rfc3339();

    let sql = format!(
        "UPDATE {table} SET docstatus = 1, modified = '{now}' WHERE name = '{sanitized_name}';"
    );
    match client.query(&sql).await {
        Ok(mut res) => {
            let docs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Document submitted successfully",
                "doc": docs.into_iter().next()
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for cancelling a document: `POST /api/v2/document/{doctype}/{name}/cancel`
pub async fn v2_cancel_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Cancel, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient cancel privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    let sanitized_name = sanitize_surrealql_string(&name);
    let now = Utc::now().to_rfc3339();

    let sql = format!(
        "UPDATE {table} SET docstatus = 2, modified = '{now}' WHERE name = '{sanitized_name}';"
    );
    match client.query(&sql).await {
        Ok(mut res) => {
            let docs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Document cancelled successfully",
                "doc": docs.into_iter().next()
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for amending a cancelled document: `POST /api/v2/document/{doctype}/{name}/amend`
pub async fn v2_amend_document(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let (doctype, name) = path.into_inner();
    let tenant_id = match req.extensions().get::<TenantId>() {
        Some(t) => t.clone(),
        None => return HttpResponse::BadRequest().body("Missing tenant context"),
    };

    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Create, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient create privileges"
        }));
    }

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    let sanitized_name = sanitize_surrealql_string(&name);

    let fetch_sql = format!("SELECT * FROM {table} WHERE name = '{sanitized_name}';");
    let mut res = match client.query(&fetch_sql).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
    let Some(mut orig_val) = records.into_iter().next() else {
        return HttpResponse::NotFound().body("Original document not found");
    };

    let doc_map = match orig_val.as_object_mut() {
        Some(m) => m,
        None => return HttpResponse::InternalServerError().body("Corrupted document format"),
    };

    let new_name = format!("{name}-1");
    let now = Utc::now().to_rfc3339();
    doc_map.insert("name".into(), serde_json::Value::String(new_name.clone()));
    doc_map.insert("docstatus".into(), serde_json::Value::Number(0.into()));
    doc_map.insert(
        "amended_from".into(),
        serde_json::Value::String(name.clone()),
    );
    doc_map.insert("creation".into(), serde_json::Value::String(now.clone()));
    doc_map.insert("modified".into(), serde_json::Value::String(now));

    let create_sql = format!("CREATE {table} CONTENT $doc;");
    match client
        .query(&create_sql)
        .bind(("doc", serde_json::Value::Object(doc_map.clone())))
        .await
    {
        Ok(mut c_res) => {
            let doc: Option<serde_json::Value> = c_res.take(0).unwrap_or(None);
            HttpResponse::Created().json(serde_json::json!({
                "message": "Document amended successfully",
                "doc": doc
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for file upload: `POST /api/v2/method/upload_file`
pub async fn upload_file_handler(
    req: HttpRequest,
    payload: web::Json<UploadFilePayload>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let raw_bytes = match hex::decode(&payload.content_base64)
        .or_else(|_| Ok::<Vec<u8>, hex::FromHexError>(payload.content_base64.as_bytes().to_vec()))
    {
        Ok(b) => b,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid file payload: {e}")),
    };

    let mut hasher = Sha256::new();
    hasher.update(&raw_bytes);
    let hash = hex::encode(hasher.finalize());
    let file_size = raw_bytes.len() as u64;

    let file_record = serde_json::json!({
        "id": hash,
        "file_name": payload.file_name,
        "content_hash": hash,
        "file_size": file_size,
        "is_private": payload.is_private.unwrap_or(false),
        "data": payload.content_base64,
        "created_at": Utc::now().to_rfc3339(),
    });

    let _ = client
        .query("CREATE drive_file CONTENT $file;")
        .bind(("file", file_record))
        .await;

    HttpResponse::Ok().json(serde_json::json!({
        "file_name": payload.file_name,
        "file_url": format!("/files/{}", hash),
        "content_hash": hash,
        "file_size": file_size,
    }))
}

/// Handler for streaming download: `GET /files/{hash}`
pub async fn download_file_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let hash = path.into_inner();
    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sanitized_hash = sanitize_surrealql_string(&hash);
    let sql = format!("SELECT * FROM drive_file WHERE content_hash = '{sanitized_hash}';");
    match client.query(&sql).await {
        Ok(mut res) => {
            let records: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            match records.into_iter().next() {
                Some(r) => {
                    let raw_data = r.get("data").and_then(|d| d.as_str()).unwrap_or_default();
                    let bytes =
                        hex::decode(raw_data).unwrap_or_else(|_| raw_data.as_bytes().to_vec());
                    HttpResponse::Ok()
                        .insert_header(("Content-Type", "application/octet-stream"))
                        .insert_header(("ETag", format!("\"{hash}\"")))
                        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
                        .body(bytes)
                }
                None => HttpResponse::NotFound().body("File not found"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for system ping: `GET /api/v2/method/ping`
pub async fn ping_handler() -> impl Responder {
    HttpResponse::Ok().json(PingResponse {
        message: "pong".into(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// Handler for user login: `POST /api/v2/method/login`
pub async fn login_handler(
    req: HttpRequest,
    body: web::Json<LoginPayload>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    if body.usr.trim().is_empty() || body.pwd.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Username and password required"
        }));
    }

    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user_id = body.usr.trim();
    let password = body.pwd.trim();

    // Query user record from SurrealDB
    let query_sql = format!("SELECT * FROM user:{}", sanitize_surrealql_string(user_id));
    let user_record: Option<serde_json::Value> = match client.query(&query_sql).await {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(_) => None,
    };

    let (is_valid, roles, full_name) = match user_record {
        Some(record) => {
            let hash_opt = record.get("password_hash").and_then(|v| v.as_str());
            let valid = match hash_opt {
                Some(hash) => verify_password(password, hash),
                None => false,
            };
            let roles: Vec<String> = record
                .get("roles")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| vec!["System Manager".into()]);
            let name = record
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or(user_id)
                .to_string();
            (valid, roles, name)
        }
        None => {
            // First-run bootstrap only when ADMIN_INITIAL_PASSWORD is explicitly provided via env
            if user_id == "Administrator" || user_id == "admin" {
                if let Ok(initial_pwd) = std::env::var("ADMIN_INITIAL_PASSWORD") {
                    if !initial_pwd.is_empty() && password == initial_pwd {
                        if let Ok(hashed) = hash_password(password) {
                            let _ = client
                                .query(format!(
                                    "CREATE user:{} SET full_name = 'Administrator', password_hash = '{}', roles = ['System Manager', 'Administrator'];",
                                    sanitize_surrealql_string(user_id),
                                    hashed
                                ))
                                .await;
                        }
                        (
                            true,
                            vec!["System Manager".into(), "Administrator".into()],
                            "Administrator".into(),
                        )
                    } else {
                        (false, vec![], String::new())
                    }
                } else {
                    (false, vec![], String::new())
                }
            } else {
                (false, vec![], String::new())
            }
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid username or password"
        }));
    }

    let claims = SessionClaims::new(user_id, &tenant_id.0, roles.clone());
    let secret = get_master_token_secret();
    let token = match issue_token(&claims, &secret) {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged In",
        "home_page": "/desk",
        "full_name": full_name,
        "user_id": user_id,
        "token": token,
        "roles": roles,
        "expires_at": claims.exp,
    }))
}

/// Handler for user logout: `POST /api/v2/method/logout`
pub async fn logout_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged Out"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_filters_to_surrealql_valid() {
        let filters = r#"[["status", "=", "Open"], ["docstatus", "=", "1"]]"#;
        let sql = compile_filters_to_surrealql(filters).unwrap();
        assert_eq!(sql, "status = \"Open\" AND docstatus = \"1\"");
    }

    #[test]
    fn test_compile_filters_rejects_sql_injection_operators() {
        let malicious_filter = r#"[["status", "= 1; DROP TABLE user; --", "Open"]]"#;
        assert_eq!(compile_filters_to_surrealql(malicious_filter), None);
    }

    #[test]
    fn test_compile_filters_rejects_malicious_fields() {
        let malicious_field = r#"[["status; DROP TABLE user; --", "=", "Open"]]"#;
        assert_eq!(compile_filters_to_surrealql(malicious_field), None);
    }

    #[test]
    fn test_compile_filters_escapes_quotes() {
        let quoted_val = r#"[["title", "=", "Test \" Injected"]]"#;
        let sql = compile_filters_to_surrealql(quoted_val).unwrap();
        assert_eq!(sql, "title = \"Test \\\" Injected\"");
    }
}
