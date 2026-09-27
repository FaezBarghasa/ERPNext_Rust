use crate::live::live_ws_handler;
use crate::routes::{create_resource, delete_resource, get_resource, list_resource};
use crate::tenant::{ConnectionPoolManager, TenantResolver};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::time::Duration;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Configures the Actix Web ERP API application.
pub fn configure_app(cfg: &mut web::ServiceConfig, pool_mgr: ConnectionPoolManager) {
    cfg.app_data(web::Data::new(pool_mgr))
        .route("/health", web::get().to(health_check))
        .route("/api/v1/live", web::get().to(live_ws_handler))
        .service(
            web::scope("/api/v1/resource")
                .wrap(TenantResolver)
                .route("/{doctype}", web::get().to(list_resource))
                .route("/{doctype}", web::post().to(create_resource))
                .route("/{doctype}/{id}", web::get().to(get_resource))
                .route("/{doctype}/{id}", web::delete().to(delete_resource)),
        );
}

/// Runs the Actix Web Server on the specified address.
pub async fn run_server(addr: &str) -> std::io::Result<()> {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(300));
    let pool_mgr_data = pool_mgr.clone();

    HttpServer::new(move || {
        App::new().configure(|cfg| configure_app(cfg, pool_mgr_data.clone()))
    })
    .bind(addr)?
    .run()
    .await
}
