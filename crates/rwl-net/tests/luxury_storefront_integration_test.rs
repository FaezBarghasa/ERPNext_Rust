use actix_web::{App, test};
use erp_cms::{CheckoutItem, CustomerCheckoutRequest};
use frappe_net::server::configure_app;
use frappe_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use rust_decimal_macros::dec;
use std::time::Duration;

#[actix_web::test]
async fn test_luxury_storefront_http_endpoints() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_app(cfg, pool_mgr, MicroTopologyConfig::default(), None)),
    )
    .await;

    // 1. Verify GET / serves the 3D luxury WebGL storefront
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let html_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(html_str.contains("LuxeGen"));
    assert!(html_str.contains("hero-canvas"));
    assert!(html_str.contains("configurator-canvas"));

    // 2. Verify GET /api/v1/storefront/products returns catalog items
    let req_products = test::TestRequest::get()
        .uri("/api/v1/storefront/products")
        .to_request();
    let resp_products = test::call_service(&app, req_products).await;
    assert!(resp_products.status().is_success());

    let products_body = test::read_body(resp_products).await;
    let products: Vec<serde_json::Value> = serde_json::from_slice(&products_body).unwrap();
    assert_eq!(products.len(), 6);
    assert_eq!(products[0]["id"], "CHRONOS-01");

    // 3. Verify POST /api/v1/storefront/checkout executes atomic checkout with balanced GL
    let checkout_req = CustomerCheckoutRequest {
        checkout_id: "CHK-TEST-001".into(),
        customer_id: "CUST-VIP-007".into(),
        items: vec![CheckoutItem {
            item_code: "CHRONOS-01".into(),
            description: "Chronos-01 Kinetic Tourbillon".into(),
            qty: dec!(1),
            unit_price: dec!(14500.00),
            available_stock: dec!(4),
        }],
        tax_rate_percent: dec!(8.0),
        receivable_account: "tab_account:debtors".into(),
        revenue_account: "tab_account:sales".into(),
        tax_account: "tab_account:tax".into(),
    };

    let req_checkout = test::TestRequest::post()
        .uri("/api/v1/storefront/checkout")
        .set_json(&checkout_req)
        .to_request();
    let resp_checkout = test::call_service(&app, req_checkout).await;
    assert!(resp_checkout.status().is_success());

    let checkout_body = test::read_body(resp_checkout).await;
    let result: serde_json::Value = serde_json::from_slice(&checkout_body).unwrap();
    assert_eq!(result["is_success"], true);
    assert_eq!(result["grand_total"], "15660.00");
}
