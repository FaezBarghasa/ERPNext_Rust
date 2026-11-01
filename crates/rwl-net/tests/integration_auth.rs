use actix_web::http::StatusCode;
use actix_web::{App, test};
use rwl_net::server::configure_app;
use rwl_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use std::time::Duration;

#[actix_web::test]
async fn test_metrics_endpoint_returns_rwl_prefix() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr, topology, None)),
    )
    .await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(
        text.contains("rwl_http_requests_total"),
        "Metrics must contain canonical rwl_http_requests_total"
    );
    assert!(
        text.contains("rwl_uptime_seconds"),
        "Metrics must contain canonical rwl_uptime_seconds"
    );
    assert!(
        !text.contains("rustnext_http_requests_total"),
        "Duplicate rustnext_ metric keys must be pruned"
    );
}

#[actix_web::test]
async fn test_health_check_returns_ok() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr, topology, None)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_forged_token_is_rejected() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr, topology, None)),
    )
    .await;

    // Send a forged bearer token to document API
    let req = test::TestRequest::get()
        .uri("/api/v2/document/Customer")
        .insert_header(("Authorization", "Bearer forged.invalid.token.here"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Without valid authorization context or missing db, returns 401 or 500/403, never 200
    assert_ne!(resp.status(), StatusCode::OK);
}

