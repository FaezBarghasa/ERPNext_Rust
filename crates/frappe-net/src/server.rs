//! Production Hypermedia Application Server (`frappe-net::server`).
//!
//! Provides:
//! - Multi-Persona Desktop & Portal SSR Shells (`/desk`, `/portal`, `/worker`, `/factory`, `/approvals`, `/admin`)
//! - REST API v1 & v2 Document/RPC Routers with Tenant Isolation, Typestate Lifecycle and RBAC
//! - SurrealDB-backed Dynamic Storefront, Atomic Checkout, and Payment Webhook Receivers
//! - Content-Addressable Storage (CAS) Upload & Download Streams
//! - Visual CMS Builder Persistence API
//! - Tracing Observability, Token-Bucket Rate Limiter, and ACME Security Gateway

use crate::live::live_ws_handler;
use crate::middleware::auth::SecurityContext;
use crate::rate_limit::RateLimitMiddleware;
use crate::routes::{create_resource, delete_resource, get_resource, list_resource};
use crate::tenant::{
    AcmeGateway, ConnectionPoolManager, MicroTopologyConfig, TenantId, TenantResolver,
};
use crate::v2_routes::{
    download_file_handler, login_handler, logout_handler, ping_handler, upload_file_handler,
    v2_amend_document, v2_cancel_document, v2_create_document, v2_delete_document, v2_get_document,
    v2_list_document, v2_submit_document, v2_update_document,
};
use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Compress,
    middleware::Logger, middleware::NormalizePath, web,
};
use desk_components::{get_desk_workspaces, render_desk_shell_html};
use std::time::Duration;
use tracing::{info, instrument};

#[instrument]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "runtime": "tokio+surrealdb",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn desk_handler(req: HttpRequest) -> impl Responder {
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
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Field Worker".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Warehouse Scanner", &user_name))
}

async fn factory_handler(req: HttpRequest) -> impl Responder {
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Machine Operator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Shopfloor MES", &user_name))
}

async fn approvals_handler(req: HttpRequest) -> impl Responder {
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Lead Approver".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Approval Deck", &user_name))
}

async fn admin_handler(req: HttpRequest) -> impl Responder {
    let user_name = req
        .extensions()
        .get::<SecurityContext>()
        .map(|ctx| ctx.claims.sub.clone())
        .unwrap_or_else(|| "Administrator".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_desk_shell_html("Admin Cockpit", &user_name))
}

async fn list_desk_workspaces_handler() -> impl Responder {
    HttpResponse::Ok().json(get_desk_workspaces())
}

async fn storefront_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::luxury_storefront::render_luxury_storefront_html())
}

#[instrument(skip(req, pool_mgr))]
async fn storefront_products_handler(
    req: HttpRequest,
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

    match client.query("SELECT * FROM product;").await {
        Ok(mut res) => {
            let products: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
            if products.is_empty() {
                let catalog = erp_cms::luxury_storefront::get_luxury_catalog();
                for item in &catalog {
                    if let Ok(item_val) = serde_json::to_value(item) {
                        let _ = client
                            .query(format!(
                                "CREATE product:{} CONTENT $item;",
                                item.id.replace('-', "_")
                            ))
                            .bind(("item", item_val))
                            .await;
                    }
                }
                HttpResponse::Ok().json(catalog)
            } else {
                HttpResponse::Ok().json(products)
            }
        }
        Err(_) => {
            let catalog = erp_cms::luxury_storefront::get_luxury_catalog();
            for item in &catalog {
                if let Ok(item_val) = serde_json::to_value(item) {
                    let _ = client
                        .query(format!(
                            "CREATE product:{} CONTENT $item;",
                            item.id.replace('-', "_")
                        ))
                        .bind(("item", item_val))
                        .await;
                }
            }
            HttpResponse::Ok().json(catalog)
        }
    }
}

#[instrument(skip(req, pool_mgr, payload))]
async fn storefront_checkout_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<erp_cms::CustomerCheckoutRequest>,
) -> impl Responder {
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

#[instrument(skip(req, pool_mgr, payload))]
async fn stripe_webhook_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<serde_json::Value>,
) -> impl Responder {
    let tenant_id = req
        .extensions()
        .get::<TenantId>()
        .cloned()
        .unwrap_or_else(|| TenantId("default".into()));

    let event = payload.into_inner();
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
    let slug = path.into_inner();
    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let sql = format!("SELECT * FROM cms_page_draft:{};", slug.replace('-', "_"));
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
    let slug = path.into_inner();
    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let doc = payload.into_inner();
    let sql = format!(
        "CREATE cms_page_draft:{} CONTENT $doc;",
        slug.replace('-', "_")
    );
    match client.query(&sql).bind(("doc", doc.clone())).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page draft saved",
            "slug": slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn cms_publish_page_handler(
    path: web::Path<String>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder {
    let slug = path.into_inner();
    let client = match pool_mgr
        .get_or_initialize_client(&TenantId("default".into()))
        .await
    {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let fetch_sql = format!("SELECT * FROM cms_page_draft:{};", slug.replace('-', "_"));
    let mut res = match client.query(&fetch_sql).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let record: Option<serde_json::Value> = res.take(0).unwrap_or(None);
    let Some(draft) = record else {
        return HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Draft not found to publish"}));
    };

    let pub_sql = format!(
        "CREATE cms_page_published:{} CONTENT $doc;",
        slug.replace('-', "_")
    );
    match client.query(&pub_sql).bind(("doc", draft)).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Page published successfully",
            "slug": slug
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn templates_portal_handler() -> impl Responder {
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
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(erp_cms::generate_sitemap_xml())
}

async fn robots_txt_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(erp_cms::generate_robots_txt())
}

async fn list_templates_api_handler() -> impl Responder {
    HttpResponse::Ok().json(erp_cms::list_template_suites())
}

async fn template_manifest_api_handler(path: web::Path<String>) -> impl Responder {
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
    analytics.record_page_view(payload.into_inner());
    HttpResponse::Ok().json(serde_json::json!({
        "status": "recorded"
    }))
}

async fn get_analytics_summary_handler(
    analytics: web::Data<erp_cms::EdgeAnalyticsEngine>,
) -> impl Responder {
    HttpResponse::Ok().json(analytics.compute_summary())
}

// ---------------------------------------------------------------------------
// Webhook Subscription Management Handlers
// ---------------------------------------------------------------------------

#[instrument(skip(req, pool_mgr, payload))]
async fn create_webhook_subscription_handler(
    req: HttpRequest,
    pool_mgr: web::Data<ConnectionPoolManager>,
    payload: web::Json<frappe_framework::WebhookSubscription>,
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
// WASI Plugin Execution Handler
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct RunPluginPayload {
    wat_code: String,
    fuel_limit: Option<u64>,
}

async fn run_plugin_handler(payload: web::Json<RunPluginPayload>) -> impl Responder {
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

/// Configures the Actix Web ERP API application.
pub fn configure_app(
    cfg: &mut web::ServiceConfig,
    pool_mgr: ConnectionPoolManager,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
) {
    cfg.app_data(web::Data::new(pool_mgr))
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
        .route("/health", web::get().to(health_check))
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
        // V2 Authentication, RPC System Methods, Webhooks & Storage
        .route("/api/v2/method/login", web::post().to(login_handler))
        .route("/api/v2/method/logout", web::post().to(logout_handler))
        .route("/api/v2/method/ping", web::get().to(ping_handler))
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
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
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
