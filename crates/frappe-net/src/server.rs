use crate::live::live_ws_handler;
use crate::routes::{create_resource, delete_resource, get_resource, list_resource};
use crate::tenant::{AcmeGateway, ConnectionPoolManager, MicroTopologyConfig, TenantResolver};
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use std::time::Duration;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn storefront_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(erp_cms::luxury_storefront::render_luxury_storefront_html())
}

async fn storefront_products_handler() -> impl Responder {
    HttpResponse::Ok().json(erp_cms::luxury_storefront::get_luxury_catalog())
}

async fn storefront_checkout_handler(
    payload: web::Json<erp_cms::CustomerCheckoutRequest>,
) -> impl Responder {
    match erp_cms::AtomicCheckoutEngine::process_checkout(&payload.into_inner()) {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string(),
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

/// Configures the Actix Web ERP API application.
pub fn configure_app(
    cfg: &mut web::ServiceConfig,
    pool_mgr: ConnectionPoolManager,
    topology: MicroTopologyConfig,
    acme_gateway: Option<AcmeGateway>,
) {
    cfg.app_data(web::Data::new(pool_mgr))
        .app_data(web::PayloadConfig::new(topology.max_payload_bytes))
        .route("/", web::get().to(storefront_handler))
        .route("/storefront", web::get().to(storefront_handler))
        .route("/health", web::get().to(health_check))
        .route("/api/v1/live", web::get().to(live_ws_handler))
        .route(
            "/api/v1/storefront/products",
            web::get().to(storefront_products_handler),
        )
        .route(
            "/api/v1/storefront/checkout",
            web::post().to(storefront_checkout_handler),
        )
        .service(
            web::scope("/api/v1/resource")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(list_resource))
                .route("/{doctype}", web::post().to(create_resource))
                .route("/{doctype}/{id}", web::get().to(get_resource))
                .route("/{doctype}/{id}", web::delete().to(delete_resource)),
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

    let mut server = HttpServer::new(move || {
        let topo = topology_clone.clone();
        let gw = gateway_clone.clone();
        App::new().configure(|cfg| configure_app(cfg, pool_mgr_data.clone(), topo, gw))
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
