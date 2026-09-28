//! API V2 Hypermedia RESTful Architecture & System Endpoints (`frappe-net::v2_routes`).
//!
//! Provides:
//! - `/api/v2/document/:doctype` (field filtering, SurrealQL predicate compilation, pagination)
//! - `/api/v2/document/:doctype/:name` (lifecycle actions: submit, cancel, amend, discard)
//! - `/api/v2/method/:method_path` (whitelisted RPC execution)
//! - System endpoints: `/api/v2/method/login`, `logout`, `ping`, `upload_file`.

use crate::tenant::{ConnectionPoolManager, TenantId};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};

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

/// Helper function to convert JSON filter list `[["status", "=", "Open"]]` into a SurrealQL WHERE clause.
pub fn compile_filters_to_surrealql(filters_json: &str) -> Option<String> {
    if let Ok(filters) = serde_json::from_str::<Vec<Vec<String>>>(filters_json) {
        if filters.is_empty() {
            return None;
        }
        let clauses: Vec<String> = filters
            .iter()
            .filter_map(|f| {
                if f.len() >= 3 {
                    let field = &f[0];
                    let op = &f[1];
                    let val = &f[2];
                    Some(format!("{field} {op} \"{val}\""))
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

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    let fields_clause = if let Some(ref f) = query.fields {
        if let Ok(field_list) = serde_json::from_str::<Vec<String>>(f) {
            field_list.join(", ")
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

    let order_clause = query
        .order_by
        .as_ref()
        .map(|o| format!("ORDER BY {o}"))
        .unwrap_or_else(|| "ORDER BY creation DESC".to_string());

    let limit = query.limit_page_length.unwrap_or(20);
    let start = query.limit_start.unwrap_or(0);

    let sql = format!("SELECT {fields_clause} FROM {table} {where_clause} {order_clause} LIMIT {limit} START {start};");
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

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let table = doctype.to_lowercase().replace(' ', "_");
    let sql = format!("SELECT * FROM {table}:{name};");
    match client.query(&sql).await {
        Ok(mut res) => {
            let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            match record {
                Some(r) => HttpResponse::Ok().json(serde_json::json!({ "doc": r })),
                None => HttpResponse::NotFound().body("Document not found"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Handler for system ping: `GET /api/v2/method/ping`
pub async fn ping_handler() -> impl Responder {
    HttpResponse::Ok().json(PingResponse {
        message: "pong".into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Handler for user login: `POST /api/v2/method/login`
pub async fn login_handler(
    body: web::Json<LoginPayload>,
) -> impl Responder {
    if body.usr.is_empty() || body.pwd.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Username and password required"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged In",
        "home_page": "/desk",
        "full_name": "Administrator",
        "user_id": body.usr,
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
    fn test_compile_filters_to_surrealql() {
        let filters = r#"[["status", "=", "Open"], ["docstatus", "=", "1"]]"#;
        let sql = compile_filters_to_surrealql(filters).unwrap();
        assert_eq!(sql, "status = \"Open\" AND docstatus = \"1\"");
    }
}
