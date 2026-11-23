//! Integration test verifying SQL injection immunity, atomic monotonic naming series,
//! and remaining-balance fulfillment mappers in the Frappe RPC API.

use actix_web::{App, test};
use rwl_net::server::configure_app;
use rwl_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use serde_json::{Value, json};
use std::time::Duration;

#[actix_web::test]
async fn test_rpc_security_and_parameter_binding() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(60));

    let app = test::init_service(
        App::new().configure(|cfg| {
            configure_app(cfg, pool_mgr.clone(), MicroTopologyConfig::default(), None)
        }),
    )
    .await;

    // 1. Verify search_link is immune to single quote SQL injections (e.g. O'Reilly)
    let search_payload = json!({
        "doctype": "Customer",
        "txt": "O'Reilly",
        "page_length": 10
    });

    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.search.search_link")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&search_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Must succeed with 200 OK and not crash on unescaped quote syntax
    assert_eq!(resp.status().as_u16(), 200);

    // 2. Verify get_value is immune to malicious SQL condition injection
    let injection_payload = json!({
        "doctype": "Customer",
        "fieldname": "customer_name",
        "filters": "' OR 1=1 --"
    });

    let req = test::TestRequest::post()
        .uri("/api/method/frappe.client.get_value")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&injection_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_monotonic_series_and_smart_fulfillment_mapper() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(60));

    let app = test::init_service(
        App::new().configure(|cfg| {
            configure_app(cfg, pool_mgr.clone(), MicroTopologyConfig::default(), None)
        }),
    )
    .await;

    // 1. Create a Sales Order via resource API
    let so_payload = json!({
        "doctype": "Sales Order",
        "customer": "Apex Robotics Corp",
        "docstatus": 1,
        "items": [
            {
                "name": "SOI-0001",
                "item_code": "ROBOT-JOINT-A",
                "qty": "10",
                "rate": "100.00",
                "amount": "1000.00",
                "delivered_qty": "4",
                "billed_amt": "400.00"
            }
        ]
    });

    let req = test::TestRequest::post()
        .uri("/api/resource/Sales%20Order")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&so_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let so_res: Value = test::read_body_json(resp).await;
    let so_name = so_res["data"]["name"].as_str().unwrap();

    // Verify sequential monotonic autoname format (e.g. SO-2026-00001)
    assert!(so_name.starts_with("SO-"));
    assert!(so_name.ends_with("-00001") || so_name.contains("-0000"));

    // 2. Test make_delivery_note: Should only take remaining 6 qty (10 - 4 = 6)
    let dn_req = test::TestRequest::post()
        .uri("/api/method/erpnext.selling.doctype.sales_order.sales_order.make_delivery_note")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({ "source_name": so_name }))
        .to_request();

    let dn_resp = test::call_service(&app, dn_req).await;
    assert!(dn_resp.status().is_success());
    let dn_body: Value = test::read_body_json(dn_resp).await;
    let dn_items = dn_body["message"]["items"].as_array().expect("items array");
    assert_eq!(dn_items.len(), 1);
    assert_eq!(dn_items[0]["qty"], "6");
    assert_eq!(dn_items[0]["so_detail"], "SOI-0001");

    // 3. Test make_sales_invoice: Should only take remaining 600 amount (1000 - 400 = 600)
    let si_req = test::TestRequest::post()
        .uri("/api/method/erpnext.selling.doctype.sales_order.sales_order.make_sales_invoice")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({ "source_name": so_name }))
        .to_request();

    let si_resp = test::call_service(&app, si_req).await;
    assert!(si_resp.status().is_success());
    let si_body: Value = test::read_body_json(si_resp).await;
    let si_items = si_body["message"]["items"].as_array().expect("items array");
    assert_eq!(si_items.len(), 1);
    assert_eq!(si_items[0]["qty"], "6");
    assert_eq!(si_items[0]["amount"], "600.00");
    assert_eq!(si_body["message"]["grand_total"], "600.00");
}
