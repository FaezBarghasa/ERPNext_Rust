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
    assert!(
        body["intent_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("PI-"))
    );
    assert!(
        body["gateway_reference"]
            .as_str()
            .is_some_and(|r| r.starts_with("CAP_MOCK_"))
    );
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

#[actix_web::test]
async fn test_gui_shell_multi_persona_and_views_integration() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Enterprise Desk Home View
    let req = test::TestRequest::get().uri("/desk").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("RustNext Enterprise Desk"));
    assert!(html.contains("Accounting & Finance"));
    assert!(html.contains("Stock & Inventory"));

    // 2. Generic DocType List View
    let req = test::TestRequest::get()
        .uri("/desk/list/sales_invoice")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("sales invoice List"));
    assert!(html.contains("RustNext Desk"));

    // 3. Document Interactive Form View
    let req = test::TestRequest::get()
        .uri("/desk/form/sales_invoice/ACC-SINV-2026-8888")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("ACC-SINV-2026-8888"));
    assert!(html.contains("sales invoice"));

    // 4. Customer Portal Persona
    let req = test::TestRequest::get().uri("/portal").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Customer Portal"));

    // 5. Warehouse Worker Kiosk Persona
    let req = test::TestRequest::get().uri("/worker").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Floor Terminal Kiosk"));

    // 6. Shopfloor MES Operator Cockpit Persona
    let req = test::TestRequest::get().uri("/factory").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("SCADA Floor Cockpit"));

    // 7. Executive Approvals Hub Persona
    let req = test::TestRequest::get().uri("/approvals").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Executive Approvals Hub"));

    // 8. Systems Administrator Cockpit Persona
    let req = test::TestRequest::get().uri("/admin").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Planetary Operational Cockpit"));

    // 9. PWA Web Manifest & Service Worker
    let req = test::TestRequest::get().uri("/manifest.json").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get().uri("/sw.js").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_v2_metadata_introspection_api() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. List all metadata schemas
    let req = test::TestRequest::get()
        .uri("/api/v2/meta")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let doctypes = body
        .get("doctypes")
        .and_then(|v| v.as_array())
        .expect("doctypes array");
    assert!(doctypes.iter().any(|d| d.as_str() == Some("Sales Invoice")));
    assert!(doctypes.iter().any(|d| d.as_str() == Some("Item")));

    // 2. Introspect sales_invoice metadata
    let req = test::TestRequest::get()
        .uri("/api/v2/meta/sales_invoice")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let schema: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        schema.get("name").and_then(|v| v.as_str()),
        Some("Sales Invoice")
    );
    let fields = schema
        .get("fields")
        .and_then(|v| v.as_array())
        .expect("fields array");
    assert!(
        fields
            .iter()
            .any(|f| f.get("fieldname").and_then(|v| v.as_str()) == Some("customer"))
    );
    assert!(
        fields
            .iter()
            .any(|f| f.get("fieldname").and_then(|v| v.as_str()) == Some("grand_total"))
    );

    // 3. Introspect item metadata
    let req = test::TestRequest::get()
        .uri("/api/v2/meta/item")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let item_schema: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        item_schema.get("name").and_then(|v| v.as_str()),
        Some("Item")
    );
    let item_fields = item_schema
        .get("fields")
        .and_then(|v| v.as_array())
        .expect("item fields array");
    assert!(
        item_fields
            .iter()
            .any(|f| f.get("fieldname").and_then(|v| v.as_str()) == Some("item_code"))
    );
}

#[actix_web::test]
async fn test_v2_invoice_settlement_reconciliation() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Create a Sales Invoice
    let inv_payload = json!({
        "name": "ACC-SINV-2026-RECON1",
        "customer": "Global Corp Ltd",
        "grand_total": 5000.0,
        "outstanding_amount": 5000.0,
        "paid_amount": 0.0,
        "status": "Unpaid"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/document/sales_invoice")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&inv_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 2. Post a Payment Charge for this invoice
    let charge_payload = json!({
        "doctype": "Sales Invoice",
        "docname": "ACC-SINV-2026-RECON1",
        "customer": "Global Corp Ltd",
        "amount": 5000.0,
        "currency": "USD",
        "payment_token": "tok_visa_enterprise",
        "payment_method": "CreditCard"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/payments/charge")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .set_json(&charge_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 3. Fetch invoice and verify outstanding_amount is 0.0 and status is Paid
    let req = test::TestRequest::get()
        .uri("/api/v2/document/sales_invoice/ACC-SINV-2026-RECON1")
        .insert_header(("X-Frappe-Site-Name", "default"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let fetched_doc: serde_json::Value = test::read_body_json(resp).await;
    let data = fetched_doc
        .get("data")
        .or_else(|| fetched_doc.get("doc"))
        .expect("doc data");
    assert_eq!(data.get("status").and_then(|v| v.as_str()), Some("Paid"));
    let out_amt = data
        .get("outstanding_amount")
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                s.parse::<f64>().ok()
            } else {
                v.as_f64()
            }
        })
        .unwrap_or(-1.0);
    assert_eq!(out_amt, 0.0);
}

#[actix_web::test]
async fn test_v2_streaming_ott_endpoints_lifecycle() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(300));
    let topology = MicroTopologyConfig::default();

    let app = test::init_service(
        App::new().configure(|cfg| configure_app(cfg, pool_mgr.clone(), topology.clone(), None)),
    )
    .await;

    // 1. Deliver HLS Master Playlist
    let req = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-demo-99/master.m3u8")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers().get("Content-Type").unwrap(),
        "application/vnd.apple.mpegurl"
    );
    let master_body = test::read_body(resp).await;
    let master_str = String::from_utf8(master_body.to_vec()).unwrap();
    assert!(master_str.contains("#EXTM3U"));
    assert!(master_str.contains("#EXT-X-STREAM-INF:BANDWIDTH=4500000"));
    assert!(master_str.contains("vid-demo-99/1080p/index.m3u8"));

    // 2. Deliver HLS Media Playlist for 1080p variant
    let req = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-demo-99/1080p/index.m3u8")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let media_body = test::read_body(resp).await;
    let media_str = String::from_utf8(media_body.to_vec()).unwrap();
    assert!(media_str.contains("#EXT-X-TARGETDURATION:4"));
    assert!(media_str.contains("1080p/seg_0.ts?token="));

    // 3. Deliver Segment (MPEG-TS binary)
    let req = test::TestRequest::get()
        .uri("/api/v2/streaming/videos/vid-demo-99/1080p/seg_0.ts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("Content-Type").unwrap(), "video/MP2T");
    let seg_body = test::read_body(resp).await;
    assert_eq!(seg_body[0], 0x47); // MPEG-TS Sync Byte

    // 4. RTMP Publish Webhook Auth (Unauthorized rejection)
    let bad_auth_payload = serde_json::json!({
        "app": "live",
        "name": "unauthorized_stream_key_999",
        "addr": "192.168.1.100"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/rtmp/auth")
        .set_json(&bad_auth_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::FORBIDDEN);

    // 5. RTMP Publish Webhook Auth (Authorized)
    let good_auth_payload = serde_json::json!({
        "app": "live",
        "name": "live_stream_key_123",
        "addr": "192.168.1.100",
        "clientid": "conn_obs_1"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/rtmp/auth")
        .set_json(&good_auth_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let auth_res: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(auth_res.get("status").and_then(|v| v.as_str()), Some("authorized"));
    assert_eq!(auth_res.get("channel_id").and_then(|v| v.as_str()), Some("live_main"));

    // 6. RTMP Done Webhook (Teardown)
    let done_payload = serde_json::json!({
        "name": "live_stream_key_123"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/rtmp/done")
        .set_json(&done_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 7. Resumable Chunk Upload
    let chunk_payload = serde_json::json!({
        "upload_id": "up_sample_video",
        "offset": 0,
        "total_bytes": 10,
        "chunk_base64": "48656c6c6f576f726c64" // "HelloWorld" in hex
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/upload/chunk")
        .set_json(&chunk_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let chunk_res: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(chunk_res.get("status").and_then(|v| v.as_str()), Some("complete"));
    assert!(chunk_res.get("sha256").is_some());

    // 8. Playback Progress Telemetry & Concurrency Heartbeat
    let progress_payload = serde_json::json!({
        "user_id": "usr_viewer_ott",
        "media_id": "vid-demo-99",
        "session_id": "sess_stream_101",
        "playhead_sec": 120,
        "duration_sec": 7200,
        "buffered_sec": 30,
        "is_paused": false
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/streaming/playback/progress")
        .set_json(&progress_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let prog_res: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(prog_res.get("status").and_then(|v| v.as_str()), Some("recorded"));
    assert_eq!(prog_res.get("playhead_sec").and_then(|v| v.as_u64()), Some(120));

    // 9. HTML5 Player Studio Page
    let req = test::TestRequest::get()
        .uri("/studio/player?v=vid-demo-99")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("Content-Type").unwrap(), "text/html; charset=utf-8");
    let html_body = test::read_body(resp).await;
    let html_str = String::from_utf8(html_body.to_vec()).unwrap();
    assert!(html_str.contains("RustNext OTT Video Player Studio"));
    assert!(html_str.contains("/api/v2/streaming/videos/vid-demo-99/master.m3u8"));
}
