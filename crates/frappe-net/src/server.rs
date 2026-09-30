//! Production Hypermedia Application Server (`frappe-net::server`).
//!
//! Provides:
//! - Multi-Persona Desktop & Portal SSR Shells (`/desk`, `/portal`, `/worker`, `/factory`, `/approvals`, `/admin`)
//! - REST API v1 & v2 Document/RPC Routers with Tenant Isolation, Typestate Lifecycle and RBAC
//! - SurrealDB-backed Dynamic Storefront, Atomic Checkout, and Payment Webhook Receivers
//! - Content-Addressable Storage (CAS) Upload & Download Streams
//! - Visual CMS Builder Persistence API with SQL Injection Hardening
//! - Tracing Observability, Prometheus Metrics, Kubernetes Health Probes (`/healthz/live`, `/healthz/ready`), Token-Bucket Rate Limiter, and ACME Security Gateway

use crate::live::live_ws_handler;
use crate::middleware::auth::SecurityContext;
use crate::rate_limit::RateLimitMiddleware;
use crate::routes::{create_resource, delete_resource, get_resource, list_resource};
use crate::tenant::{
    AcmeGateway, ConnectionPoolManager, MicroTopologyConfig, TenantId, TenantResolver,
};
use crate::v2_routes::{
    DynamicRbacState, accounting_journal_entry_handler, accounting_trial_balance_handler,
    admin_action_handler, admin_create_ip_rule_handler, admin_create_role_handler,
    admin_create_user_handler, admin_delete_ip_rule_handler, admin_delete_user_handler,
    admin_get_audit_logs_handler, admin_get_permissions_handler,
    admin_get_user_effective_permissions_handler, admin_list_ip_rules_handler,
    admin_list_lockouts_handler, admin_list_roles_handler, admin_list_users_handler,
    admin_query_handler, admin_status_handler, admin_unlock_target_handler,
    admin_update_permission_handler, admin_update_user_roles_handler, auth_forgot_password_handler,
    auth_mfa_activate_handler, auth_mfa_disable_handler, auth_mfa_enroll_handler,
    auth_refresh_token_handler, auth_reset_password_handler, cms_create_taxonomy_handler,
    cms_generate_seo_handler, cms_list_media_handler, cms_list_taxonomies_handler,
    cms_upload_media_handler, crm_convert_quotation_handler, download_file_handler,
    export_dataset_handler, h3_stream_file_handler, h3_stream_telemetry_handler,
    hr_process_payroll_handler, inventory_stock_balance_handler, inventory_stock_entry_handler,
    login_handler, logout_handler, notifications_dispatch_handler, notifications_get_inbox_handler, ping_handler,
    quic_status_handler, render_invoice_handler, render_receipt_handler,
    report_pivot_table_handler, trade_add_wishlist_item_handler, trade_apply_coupon_handler,
    trade_calculate_shipping_handler, trade_create_coupon_handler, trade_get_wishlist_handler,
    trade_list_coupons_handler, trade_list_reviews_handler, trade_list_shipping_zones_handler,
    trade_moderate_review_handler, trade_order_transition_handler,
    trade_remove_wishlist_item_handler, trade_rma_submit_handler, trade_submit_review_handler,
    upload_file_handler, v2_amend_document, v2_cancel_document, v2_create_document,
    v2_delete_document, v2_get_document, v2_list_document, v2_submit_document, v2_update_document,
    webhooks_dispatch_test_handler, workflow_evaluate_handler, workflow_version_history_handler,
    workflow_version_rollback_handler,
};
use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Compress,
    middleware::Logger, middleware::NormalizePath, web,
};
use desk_components::{get_desk_workspaces, render_desk_shell_html};
use frappe_framework::WebhookDispatcher;
use frappe_meta::rbac::{Permission, check_permission};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, instrument};

static TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// Sanitizes a CMS page slug to prevent SQL injection and directory traversal.
#[must_use]
pub fn sanitize_slug(slug: &str) -> Option<String> {
    let trimmed = slug.trim();
    if !trimmed.is_empty()
        && trimmed.len() <= 128
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(trimmed.replace('-', "_"))
    } else {
        None
    }
}

#[instrument]
async fn health_check() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "runtime": "tokio+surrealdb",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Kubernetes liveness probe: `GET /healthz/live`
#[instrument]
async fn liveness_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Kubernetes readiness probe: `GET /healthz/ready`
#[instrument(skip(pool_mgr))]
async fn readiness_handler(pool_mgr: web::Data<ConnectionPoolManager>) -> impl Responder {
    match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(client) => match client.query("SELECT 1;").await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                "status": "ready",
                "db": "connected",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })),
            Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_ready",
                "db": "unavailable",
                "error": e.to_string(),
            })),
        },
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "db": "connection_failed",
            "error": e.to_string(),
        })),
    }
}

/// Prometheus metrics exposition: `GET /metrics`
async fn metrics_handler() -> impl Responder {
    let total_reqs = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let uptime_secs = START_TIME.elapsed().as_secs();

    let metrics_text = format!(
        "# HELP rustnext_http_requests_total Total number of HTTP requests processed.\n\
         # TYPE rustnext_http_requests_total counter\n\
         rustnext_http_requests_total {}\n\
         # HELP rustnext_uptime_seconds Total runtime uptime in seconds.\n\
         # TYPE rustnext_uptime_seconds gauge\n\
         rustnext_uptime_seconds {}\n\
         # HELP rustnext_active_tenant_sessions Active cached tenant sessions count.\n\
         # TYPE rustnext_active_tenant_sessions gauge\n\
         rustnext_active_tenant_sessions 1\n",
        total_reqs, uptime_secs
    );

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text)
}

async fn desk_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Enterprise Desk", &user_name))
}

async fn portal_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Customer".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Customer Portal", &user_name))
}

async fn worker_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Operator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(desk_app::render_worker_kiosk_html(&user_name))
}

async fn factory_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Factory Manager".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("SCADA Floor Cockpit", &user_name))
}

async fn approvals_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Executive".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html(
            "Executive Approvals Hub",
            &user_name,
        ))
}

async fn admin_handler(req: HttpRequest) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(desk_app::render_admin_cockpit_html(&user_name))
}

async fn storefront_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::render_luxury_storefront_html())
}

async fn list_desk_workspaces_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "workspaces": get_desk_workspaces()
    }))
}

async fn storefront_products_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(erp_cms::get_luxury_catalog())
}

#[instrument(skip(req, pool_mgr, payload))]
async fn storefront_checkout_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<erp_cms::CustomerCheckoutRequest>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let checkout_req = payload.into_inner();
    match erp_cms::AtomicCheckoutEngine::process_checkout(&checkout_req) {
        Ok(result) => {
            if let Ok(client) = pool_mgr.get_or_initialize_client(&tenant_id).await
                && let Ok(invoice_val) = serde_json::to_value(&result)
            {
                let _ = client
                    .query(format!(
                        "CREATE sales_invoice:{} CONTENT $invoice;",
                        result.invoice_id.replace('-', "_")
                    ))
                    .bind(("invoice", invoice_val))
                    .await;
            }
            HttpResponse::Ok().json(result)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string(),
        })),
    }
}

#[instrument(skip(req, pool_mgr, body))]
async fn stripe_webhook_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    body: web::Bytes,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let sig_header = req
        .headers()
        .get("Stripe-Signature")
        .or_else(|| req.headers().get("X-RustNext-Signature-256"))
        .or_else(|| req.headers().get("X-Signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let stripe_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
    if !stripe_secret.is_empty()
        && !WebhookDispatcher::verify_signature(&stripe_secret, &body, sig_header)
    {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid webhook signature"
        }));
    }

    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let event: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid JSON payload: {e}")
            }));
        }
    };

    let event_type = event
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or_default();

    if (event_type == "payment_intent.succeeded" || event_type == "checkout.session.completed")
        && let Some(data) = event.get("data").and_then(|d| d.get("object"))
        && let Some(invoice_id) = data
            .get("invoice_id")
            .or_else(|| data.get("id"))
            .and_then(|i| i.as_str())
        && let Ok(client) = pool_mgr.get_or_initialize_client(&tenant_id).await
    {
        let sanitized_id = invoice_id.replace('-', "_");
        let now = chrono::Utc::now().to_rfc3339();
        let _ = client
            .query(format!(
                "UPDATE sales_invoice:{sanitized_id} SET status = 'Paid', paid_at = '{now}';"
            ))
            .await;
    }

    HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "event_type": event_type
    }))
}

async fn cms_get_page_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sql = format!("SELECT * FROM cms_page_draft:{sanitized};");
    match client.query(&sql).await {
        Ok(mut res) => {
            let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
            match record {
                Some(r) => HttpResponse::Ok().json(r),
                None => HttpResponse::NotFound()
                    .json(serde_json::json!({"error": "Page draft not found"})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn cms_save_page_handler(
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let doc = payload.into_inner();
    let sql = format!("CREATE cms_page_draft:{sanitized} CONTENT $doc;");
    match client.query(&sql).bind(("doc", doc.clone())).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page draft saved",
            "slug": raw_slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn cms_publish_page_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let raw_slug = path.into_inner();
    let sanitized = match sanitize_slug(&raw_slug) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid slug identifier"}));
        }
    };

    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let fetch_sql = format!("SELECT * FROM cms_page_draft:{sanitized};");
    let mut res = match client.query(&fetch_sql).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
    let Some(draft) = record else {
        return HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Draft not found to publish"}));
    };

    let pub_sql = format!("CREATE cms_page_published:{sanitized} CONTENT $doc;");
    match client.query(&pub_sql).bind(("doc", draft)).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page published successfully",
            "slug": raw_slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn templates_portal_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::render_template_index_html())
}

#[derive(serde::Deserialize)]
struct TemplateQuery {
    variant: Option<String>,
}

async fn template_detail_handler(
    path: web::Path<String>,
    query: web::Query<TemplateQuery>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let slug = path.into_inner();
    let variant = query.variant.as_deref();
    match erp_cms::render_template_html_with_variant(&slug, variant) {
        Some(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Template '{slug}' not found"),
            "available_templates": erp_cms::list_template_suites().into_iter().map(|s| s.slug).collect::<Vec<_>>(),
        })),
    }
}

async fn sitemap_xml_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(erp_cms::generate_sitemap_xml())
}

async fn robots_txt_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(erp_cms::generate_robots_txt())
}

async fn list_templates_api_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(erp_cms::list_template_suites())
}

async fn template_manifest_api_handler(path: web::Path<String>) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let slug = path.into_inner();
    match erp_cms::get_template_suite(&slug) {
        Some(suite) => HttpResponse::Ok().json(suite.to_theme_manifest()),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Template '{slug}' not found"),
        })),
    }
}

async fn acme_challenge_handler(
    path: web::Path<String>,
    gateway: web::Data<AcmeGateway>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let token = path.into_inner();
    match gateway.resolve_domain(&token).await {
        Some(_) => HttpResponse::Ok().body(format!("{token}.simulated_acme_key_authorization")),
        None => HttpResponse::NotFound().body("ACME Challenge Not Found"),
    }
}

// ---------------------------------------------------------------------------
// Edge Analytics Handlers
// ---------------------------------------------------------------------------

async fn record_analytics_event_handler(
    analytics: web::Data<erp_cms::EdgeAnalyticsEngine>,
    payload: web::Json<erp_cms::PageViewEvent>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    analytics.record_page_view(payload.into_inner());
    HttpResponse::Ok().json(serde_json::json!({
        "status": "recorded"
    }))
}

async fn get_analytics_summary_handler(
    analytics: web::Data<erp_cms::EdgeAnalyticsEngine>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(analytics.compute_summary())
}

// ---------------------------------------------------------------------------
// Webhook Subscription Management Handlers (RBAC Hardened)
// ---------------------------------------------------------------------------

#[instrument(skip(req, pool_mgr, payload))]
async fn create_webhook_subscription_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<frappe_framework::WebhookSubscription>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Write, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient write privileges"
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

    let sub = payload.into_inner();
    let sanitized_id = sub.id.replace('-', "_");
    let sql = format!("CREATE webhook_subscription:{sanitized_id} CONTENT $sub;");
    let sub_val = match serde_json::to_value(&sub) {
        Ok(v) => v,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    match client.query(&sql).bind(("sub", sub_val)).await {
        Ok(_) => HttpResponse::Created().json(sub),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[instrument(skip(req, pool_mgr))]
async fn list_webhook_subscriptions_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Read, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient read privileges"
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

    match client.query("SELECT * FROM webhook_subscription;").await {
        Ok(mut res) => {
            let subs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            HttpResponse::Ok().json(subs)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[instrument(skip(req, pool_mgr))]
async fn delete_webhook_subscription_handler(
    path: web::Path<String>,
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Write, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient write privileges"
        }));
    }

    let id = path.into_inner();
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let client = match pool_mgr.get_or_initialize_client(&tenant_id).await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sanitized_id = id.replace('-', "_");
    match client
        .query(format!("DELETE webhook_subscription:{sanitized_id};"))
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Subscription deleted",
            "id": id
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// WASI Plugin Execution Handler (RBAC Hardened)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct RunPluginPayload {
    wat_code: String,
    fuel_limit: Option<u64>,
}

async fn run_plugin_handler(
    req: HttpRequest,
    payload: web::Json<RunPluginPayload>,
) -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    if let Some(ctx) = req.extensions().get::<SecurityContext>()
        && !check_permission(&ctx.claims.roles, &[], Permission::Execute, 0)
    {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Permission Denied: insufficient execute privileges for plugins"
        }));
    }

    let inner = payload.into_inner();
    let fuel = inner.fuel_limit.unwrap_or(1_000_000);
    let sandbox = match frappe_framework::RealSandbox::new(fuel, 32 * 1024 * 1024) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create sandbox: {e}")
            }));
        }
    };

    match sandbox.run_wat(&inner.wat_code) {
        Ok(result) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "exit_code": result
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "error": e.to_string()
        })),
    }
}

async fn pwa_manifest_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let manifest = desk_app::PwaWebManifest::new("RustNext Enterprise", "#6366f1", "#0f172a");
    HttpResponse::Ok()
        .content_type("application/manifest+json")
        .json(manifest)
}

async fn service_worker_handler() -> impl Responder {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
    let sw_script =
        desk_app::ServiceWorkerGenerator::generate_service_worker_js(env!("CARGO_PKG_VERSION"));
    HttpResponse::Ok()
        .content_type("application/javascript")
        .insert_header(("Cache-Control", "public, max-age=0, must-revalidate"))
        .body(sw_script)
}

/// Configures the Actix Web ERP API application.
pub fn configure_app(
    cfg: &mut web::ServiceConfig,
    pool_mgr: ConnectionPoolManager,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
) {
    cfg.app_data(web::Data::new(pool_mgr))
        .app_data(web::Data::new(DynamicRbacState::default()))
        .app_data(web::Data::new(erp_cms::EdgeAnalyticsEngine::new()))
        .app_data(web::PayloadConfig::new(topology.max_payload_bytes))
        // Root storefront & Multi-Persona Shells
        .route("/", web::get().to(storefront_handler))
        .route("/desk", web::get().to(desk_handler))
        .route("/portal", web::get().to(portal_handler))
        .route("/worker", web::get().to(worker_handler))
        .route("/factory", web::get().to(factory_handler))
        .route("/approvals", web::get().to(approvals_handler))
        .route("/admin", web::get().to(admin_handler))
        .route("/storefront", web::get().to(storefront_handler))
        .route("/templates", web::get().to(templates_portal_handler))
        .route("/templates/{slug}", web::get().to(template_detail_handler))
        .route("/sitemap.xml", web::get().to(sitemap_xml_handler))
        .route("/robots.txt", web::get().to(robots_txt_handler))
        .route("/manifest.json", web::get().to(pwa_manifest_handler))
        .route("/sw.js", web::get().to(service_worker_handler))
        .route("/health", web::get().to(health_check))
        .route("/healthz/live", web::get().to(liveness_handler))
        .route("/healthz/ready", web::get().to(readiness_handler))
        .route("/metrics", web::get().to(metrics_handler))
        .route("/files/{hash}", web::get().to(download_file_handler))
        .route("/api/v1/live", web::get().to(live_ws_handler))
        .route(
            "/api/v1/desk/workspaces",
            web::get().to(list_desk_workspaces_handler),
        )
        .route(
            "/api/v1/storefront/products",
            web::get().to(storefront_products_handler),
        )
        .route(
            "/api/v1/storefront/checkout",
            web::post().to(storefront_checkout_handler),
        )
        .route(
            "/api/v1/storefront/webhooks/stripe",
            web::post().to(stripe_webhook_handler),
        )
        .route(
            "/api/v1/templates",
            web::get().to(list_templates_api_handler),
        )
        .route(
            "/api/v1/templates/{slug}/manifest",
            web::get().to(template_manifest_api_handler),
        )
        // Edge Analytics API
        .route(
            "/api/v1/analytics/event",
            web::post().to(record_analytics_event_handler),
        )
        .route(
            "/api/v1/analytics/summary",
            web::get().to(get_analytics_summary_handler),
        )
        // Universal AI Agent OAuth 2.0 Token Server
        .route(
            "/api/v1/oauth/token",
            web::post().to(crate::ai_oauth::oauth_token_handler),
        )
        .route(
            "/oauth/token",
            web::post().to(crate::ai_oauth::oauth_token_handler),
        )
        // V2 Authentication, RPC System Methods, Webhooks & Storage
        .route("/api/v2/quic/status", web::get().to(quic_status_handler))
        .route(
            "/api/v2/stream/h3/file/{hash}",
            web::get().to(h3_stream_file_handler),
        )
        .route(
            "/api/v2/stream/h3/telemetry",
            web::post().to(h3_stream_telemetry_handler),
        )
        .route("/api/v2/admin/status", web::get().to(admin_status_handler))
        .route(
            "/api/v2/admin/action/{action_id}",
            web::post().to(admin_action_handler),
        )
        .route("/api/v2/admin/query", web::post().to(admin_query_handler))
        // Dynamic User & Role Permission Management
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
        .route("/api/v2/method/login", web::post().to(login_handler))
        .route("/api/v2/method/logout", web::post().to(logout_handler))
        .route("/api/v2/method/ping", web::get().to(ping_handler))
        .route(
            "/api/v2/auth/refresh",
            web::post().to(auth_refresh_token_handler),
        )
        .route(
            "/api/v2/method/render_invoice",
            web::post().to(render_invoice_handler),
        )
        .route(
            "/api/v2/method/render_receipt",
            web::post().to(render_receipt_handler),
        )
        // Enterprise Domain Modules: Accounting, Inventory, CRM, HR
        .route(
            "/api/v2/accounting/journal_entry",
            web::post().to(accounting_journal_entry_handler),
        )
        .route(
            "/api/v2/accounting/trial_balance",
            web::get().to(accounting_trial_balance_handler),
        )
        .route(
            "/api/v2/inventory/stock_entry",
            web::post().to(inventory_stock_entry_handler),
        )
        .route(
            "/api/v2/inventory/balance/{warehouse}/{item_code}",
            web::get().to(inventory_stock_balance_handler),
        )
        .route(
            "/api/v2/crm/quotations/convert",
            web::post().to(crm_convert_quotation_handler),
        )
        .route(
            "/api/v2/hr/payroll/process",
            web::post().to(hr_process_payroll_handler),
        )
        // Wave 1 MFA & Password Security Endpoints
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
            "/api/v2/auth/forgot-password",
            web::post().to(auth_forgot_password_handler),
        )
        .route(
            "/api/v2/auth/reset-password",
            web::post().to(auth_reset_password_handler),
        )
        // Wave 1 Enterprise Audit Logging & IP Security Governance
        .route(
            "/api/v2/admin/audit-logs",
            web::get().to(admin_get_audit_logs_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules",
            web::get().to(admin_list_ip_rules_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules",
            web::post().to(admin_create_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/security/ip-rules/{rule_id}",
            web::delete().to(admin_delete_ip_rule_handler),
        )
        .route(
            "/api/v2/admin/security/lockouts",
            web::get().to(admin_list_lockouts_handler),
        )
        .route(
            "/api/v2/admin/security/unlock",
            web::post().to(admin_unlock_target_handler),
        )
        .route(
            "/api/v2/method/upload_file",
            web::post().to(upload_file_handler),
        )
        .route(
            "/api/v2/method/run_plugin",
            web::post().to(run_plugin_handler),
        )
        .route(
            "/api/v2/webhooks/subscribe",
            web::post().to(create_webhook_subscription_handler),
        )
        .route(
            "/api/v2/webhooks/subscriptions",
            web::get().to(list_webhook_subscriptions_handler),
        )
        .route(
            "/api/v2/webhooks/subscribe/{id}",
            web::delete().to(delete_webhook_subscription_handler),
        )
        // V2 Visual CMS Persistence Endpoints
        .route(
            "/api/v2/cms/page/{slug}",
            web::get().to(cms_get_page_handler),
        )
        .route(
            "/api/v2/cms/page/{slug}/save",
            web::post().to(cms_save_page_handler),
        )
        .route(
            "/api/v2/cms/page/{slug}/publish",
            web::post().to(cms_publish_page_handler),
        )
        // V2 Commerce & Promotion Endpoints
        .route(
            "/api/v2/trade/coupons",
            web::get().to(trade_list_coupons_handler),
        )
        .route(
            "/api/v2/trade/coupons",
            web::post().to(trade_create_coupon_handler),
        )
        .route(
            "/api/v2/trade/coupons/apply",
            web::post().to(trade_apply_coupon_handler),
        )
        .route(
            "/api/v2/trade/reviews",
            web::get().to(trade_list_reviews_handler),
        )
        .route(
            "/api/v2/trade/reviews",
            web::post().to(trade_submit_review_handler),
        )
        .route(
            "/api/v2/trade/reviews/moderate",
            web::post().to(trade_moderate_review_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}",
            web::get().to(trade_get_wishlist_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}/items",
            web::post().to(trade_add_wishlist_item_handler),
        )
        .route(
            "/api/v2/trade/wishlists/{user_id}/items/{item_code}",
            web::delete().to(trade_remove_wishlist_item_handler),
        )
        .route(
            "/api/v2/trade/shipping/zones",
            web::get().to(trade_list_shipping_zones_handler),
        )
        .route(
            "/api/v2/trade/shipping/calculate",
            web::post().to(trade_calculate_shipping_handler),
        )
        .route(
            "/api/v2/trade/orders/transition",
            web::post().to(trade_order_transition_handler),
        )
        .route(
            "/api/v2/trade/rma",
            web::post().to(trade_rma_submit_handler),
        )
        // V2 CMS Taxonomy, Media & SEO Endpoints
        .route(
            "/api/v2/cms/taxonomy",
            web::get().to(cms_list_taxonomies_handler),
        )
        .route(
            "/api/v2/cms/taxonomy",
            web::post().to(cms_create_taxonomy_handler),
        )
        .route("/api/v2/cms/media", web::get().to(cms_list_media_handler))
        .route(
            "/api/v2/cms/media",
            web::post().to(cms_upload_media_handler),
        )
        .route(
            "/api/v2/cms/seo/generate",
            web::post().to(cms_generate_seo_handler),
        )
        // V2 Approval Workflow & Versioning Endpoints
        .route(
            "/api/v2/workflow/evaluate",
            web::post().to(workflow_evaluate_handler),
        )
        .route(
            "/api/v2/workflow/versions/{doctype}/{docname}",
            web::get().to(workflow_version_history_handler),
        )
        .route(
            "/api/v2/workflow/versions/rollback",
            web::post().to(workflow_version_rollback_handler),
        )
        // V2 Universal Export & Pivot Reporting Endpoints
        .route(
            "/api/v2/export/data",
            web::post().to(export_dataset_handler),
        )
        .route(
            "/api/v2/reports/pivot",
            web::post().to(report_pivot_table_handler),
        )
        // V2 Omni-Channel Notifications & Webhooks Endpoints
        .route(
            "/api/v2/notifications/inbox/{user_id}",
            web::get().to(notifications_get_inbox_handler),
        )
        .route(
            "/api/v2/notifications/dispatch",
            web::post().to(notifications_dispatch_handler),
        )
        .route(
            "/api/v2/webhooks/test",
            web::post().to(webhooks_dispatch_test_handler),
        )
        // Protected V1 REST Resource API Scope
        .service(
            web::scope("/api/v1/resource")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(list_resource))
                .route("/{doctype}", web::post().to(create_resource))
                .route("/{doctype}/{id}", web::get().to(get_resource))
                .route("/{doctype}/{id}", web::delete().to(delete_resource)),
        )
        // Protected V2 Hypermedia Document API Scope
        .service(
            web::scope("/api/v2/document")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(v2_list_document))
                .route("/{doctype}", web::post().to(v2_create_document))
                .route("/{doctype}/{name}", web::get().to(v2_get_document))
                .route("/{doctype}/{name}", web::put().to(v2_update_document))
                .route("/{doctype}/{name}", web::delete().to(v2_delete_document))
                .route(
                    "/{doctype}/{name}/submit",
                    web::post().to(v2_submit_document),
                )
                .route(
                    "/{doctype}/{name}/cancel",
                    web::post().to(v2_cancel_document),
                )
                .route("/{doctype}/{name}/amend", web::post().to(v2_amend_document)),
        );

    if let Some(gateway) = acme_gateway {
        cfg.app_data(web::Data::new(gateway)).route(
            "/.well-known/acme-challenge/{token}",
            web::get().to(acme_challenge_handler),
        );
    }
}

/// Runs the Actix Web Server on the specified address with custom topology and ACME gateway options.
pub async fn run_server_with_config(
    addr: &str,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
    workers: Option<usize>,
) -> std::io::Result<()> {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(300));
    let pool_mgr_data = pool_mgr.clone();
    let topology_clone = topology.clone();
    let gateway_clone = acme_gateway.clone();

    info!(
        "Starting RustNext server on {addr} with workers: {:?}",
        workers
    );

    let mut server = HttpServer::new(move || {
        let topo = topology_clone.clone();
        let gw = gateway_clone.clone();
        App::new()
            .wrap(crate::middleware::RequestIdMiddleware::new())
            .wrap(crate::middleware::CorsMiddleware::new())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .wrap(crate::middleware::SecurityHeaders::new())
            .wrap(RateLimitMiddleware::new(300.0, 50.0))
            .configure(|cfg| configure_app(cfg, pool_mgr_data.clone(), topo, gw))
    });

    let effective_workers = workers.unwrap_or(if topology.is_micro_mode {
        1
    } else {
        num_cpus()
    });
    server = server.workers(effective_workers);

    server.bind(addr)?.run().await
}

/// Runs the Actix Web Server on the specified address with default configuration.
pub async fn run_server(addr: &str) -> std::io::Result<()> {
    run_server_with_config(addr, MicroTopologyConfig::default(), None, None).await
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_slug_valid() {
        assert_eq!(sanitize_slug("home-page"), Some("home_page".into()));
        assert_eq!(sanitize_slug("about_us_2026"), Some("about_us_2026".into()));
        assert_eq!(sanitize_slug("pricing"), Some("pricing".into()));
    }

    #[test]
    fn test_sanitize_slug_rejects_malicious() {
        assert_eq!(sanitize_slug(""), None);
        assert_eq!(sanitize_slug("home; DROP TABLE users;"), None);
        assert_eq!(sanitize_slug("../etc/passwd"), None);
        assert_eq!(sanitize_slug("foo$bar"), None);
    }
}
