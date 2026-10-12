use actix_web::{App, test};
use frappe_net::configure_app;
use frappe_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use serde_json::json;
use std::time::Duration;

#[actix_web::test]
async fn test_v2_document_lifecycle_integration() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Ping
    let req = test::TestRequest::get()
        .uri("/api/v2/method/ping")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 2. Create Document via POST /api/v2/document/quotation
    let create_payload = json!({
        "name": "QUOT-2026-0001",
        "customer": "Apex Manufacturing Ltd",
        "grand_total": 45000.0,
        "status": "Draft"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/document/quotation")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&create_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 3. Get Document via GET /api/v2/document/quotation/QUOT-2026-0001
    let req = test::TestRequest::get()
        .uri("/api/v2/document/quotation/QUOT-2026-0001")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 4. Update Document via PUT /api/v2/document/quotation/QUOT-2026-0001
    let update_payload = json!({
        "customer": "Apex Global Solutions Ltd",
        "grand_total": 48000.0
    });
    let req = test::TestRequest::put()
        .uri("/api/v2/document/quotation/QUOT-2026-0001")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&update_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 5. Submit Document via POST /api/v2/document/quotation/QUOT-2026-0001/submit
    let req = test::TestRequest::post()
        .uri("/api/v2/document/quotation/QUOT-2026-0001/submit")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 6. Attempting to delete submitted document must fail
    let req = test::TestRequest::delete()
        .uri("/api/v2/document/quotation/QUOT-2026-0001")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // 7. Cancel Document via POST /api/v2/document/quotation/QUOT-2026-0001/cancel
    let req = test::TestRequest::post()
        .uri("/api/v2/document/quotation/QUOT-2026-0001/cancel")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 8. Amend Document via POST /api/v2/document/quotation/QUOT-2026-0001/amend
    let req = test::TestRequest::post()
        .uri("/api/v2/document/quotation/QUOT-2026-0001/amend")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn test_v2_file_upload_and_download_integration() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // Upload file
    let upload_payload = json!({
        "file_name": "invoice_spec.pdf",
        "content_base64": "JVBERi0xLjQKJcTl8uXrCjEgMCBvYmoKPDwKL1R5cGUgL0NhdGFsb2c",
        "is_private": false
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/method/upload_file")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&upload_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let file_url = body.get("file_url").and_then(|u| u.as_str()).unwrap();
    let content_hash = body.get("content_hash").and_then(|h| h.as_str()).unwrap();

    assert!(file_url.starts_with("/files/"));
    assert!(!content_hash.is_empty());

    // Download file
    let req = test::TestRequest::get()
        .uri(file_url)
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers()
            .get("Cache-Control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=31536000, immutable")
    );
}

#[actix_web::test]
async fn test_v2_document_print_format_integration() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Create a Sales Invoice
    let invoice_payload = json!({
        "name": "ACC-SINV-2026-8888",
        "doctype": "Sales Invoice",
        "customer": "Apex Precision Aerospace",
        "company": "RustNext Global Industries",
        "posting_date": "2026-10-12",
        "currency": "USD",
        "grand_total": 95000.0,
        "net_total": 85000.0,
        "items": [
            {
                "item_code": "JET-TURB-01",
                "description": "High Pressure Turbine Blade Assembly",
                "qty": 10.0,
                "rate": 8500.0,
                "tax_rate": 10.0,
                "amount": 85000.0
            }
        ],
        "taxes": [
            {
                "description": "Standard Sales VAT",
                "rate": 10.0,
                "tax_amount": 8500.0
            }
        ]
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/document/sales_invoice")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&invoice_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 2. Fetch Commercial Print View via GET /api/v2/document/sales_invoice/ACC-SINV-2026-8888/print
    let req = test::TestRequest::get()
        .uri("/api/v2/document/sales_invoice/ACC-SINV-2026-8888/print")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );

    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("ACC-SINV-2026-8888"));
    assert!(html.contains("Apex Precision Aerospace"));
    assert!(html.contains("JET-TURB-01"));
    assert!(html.contains("@media print"));

    // 3. Test Universal Print via POST /api/v2/print/document
    let print_payload = json!({
        "doctype": "Sales Invoice",
        "docname": "ACC-SINV-2026-8888",
        "raw_data": invoice_payload
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/print/document")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&print_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("ACC-SINV-2026-8888"));
    assert!(html.contains("JET-TURB-01"));
}

#[actix_web::test]
async fn test_v2_payment_charge_and_webhook_integration() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Charge Payment via POST /api/v2/payments/charge
    let charge_payload = json!({
        "doctype": "Sales Invoice",
        "docname": "ACC-SINV-2026-8888",
        "customer": "Apex Precision Aerospace",
        "amount": 95000.0,
        "currency": "USD",
        "payment_token": "tok_visa_enterprise",
        "auto_reconcile": true
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/payments/charge")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&charge_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "SUCCESS");
    assert!(body["intent_id"].as_str().is_some_and(|id| id.starts_with("PI-")));
    assert!(body["gateway_reference"].as_str().is_some_and(|r| r.starts_with("CAP_MOCK_")));
    assert!(body["settlement_journal"].is_object());

    // 2. Reject Negative Charge Amount
    let invalid_charge = json!({
        "doctype": "Sales Invoice",
        "docname": "ACC-SINV-2026-8888",
        "customer": "Apex Precision Aerospace",
        "amount": -50.0,
        "currency": "USD",
        "payment_token": "tok_visa_enterprise"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/payments/charge")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&invalid_charge)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // 3. Receive Asynchronous Payment Webhook via POST /api/v2/payments/webhook
    let webhook_payload = json!({
        "event_type": "payment_intent.succeeded",
        "intent_id": "PI-MOCK-99001",
        "gateway_reference": "ch_mock_enterprise_99",
        "amount": 95000.0,
        "currency": "USD",
        "status": "Captured",
        "customer": "Apex Precision Aerospace",
        "reference_doctype": "Sales Invoice",
        "reference_docname": "ACC-SINV-2026-8888"
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/payments/webhook")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&webhook_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["received"], true);
    assert_eq!(body["status"], "PROCESSED");
}
