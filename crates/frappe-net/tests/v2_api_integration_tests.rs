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
