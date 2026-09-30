use actix_web::{App, test};
use frappe_net::server::configure_app;
use frappe_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use std::time::Duration;

#[actix_web::test]
async fn test_enterprise_hardening_routes() {
    let pool_mgr = ConnectionPoolManager::new(Duration::from_secs(60));
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_app(cfg, pool_mgr, MicroTopologyConfig::default(), None)),
    )
    .await;

    // =========================================================================
    // 1. Dynamic RBAC User & Permission Endpoints
    // =========================================================================
    let req = test::TestRequest::get()
        .uri("/api/v2/admin/users")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let users_val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(users_val["status"], "OK");
    assert!(users_val["users"].as_array().unwrap().len() >= 6);

    // =========================================================================
    // 2. Commerce: Coupons & Promotions
    // =========================================================================
    let req = test::TestRequest::get()
        .uri("/api/v2/trade/coupons")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Apply Coupon
    let apply_payload = serde_json::json!({
        "code": "WELCOME10",
        "subtotal": "100.00",
        "user_id": "cust_123",
        "items": [
            {
                "item_code": "LAPTOP-01",
                "item_group": "Electronics",
                "qty": "1",
                "unit_price": "100.00",
                "line_total": "100.00"
            }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/coupons/apply")
        .set_json(&apply_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let coupon_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(coupon_res["status"], "VALID");
    assert_eq!(coupon_res["calculation"]["discount_amount"], "10.0000");

    // =========================================================================
    // 3. Commerce: Product Reviews & Ratings
    // =========================================================================
    let review_payload = serde_json::json!({
        "item_code": "LAPTOP-01",
        "user_id": "cust_john",
        "user_display_name": "John Doe",
        "rating": 5,
        "title": "Outstanding Performance",
        "content": "Blazing fast Rust compile times!",
        "verified_purchase": true
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/reviews")
        .set_json(&review_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let review_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let review_id = review_res["review"]["id"].as_str().unwrap();

    // Moderate Review
    let mod_payload = serde_json::json!({
        "review_id": review_id,
        "status": "approved"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/reviews/moderate")
        .set_json(&mod_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // =========================================================================
    // 4. Commerce: Wishlist
    // =========================================================================
    let wish_payload = serde_json::json!({
        "item_code": "LAPTOP-01",
        "priority": 2,
        "desired_price": "1499.00",
        "notes": "Alert on price drop"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/wishlists/cust_john/items")
        .set_json(&wish_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/api/v2/trade/wishlists/cust_john")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // =========================================================================
    // 5. Commerce: Shipping Zones & Rates
    // =========================================================================
    let req = test::TestRequest::get()
        .uri("/api/v2/trade/shipping/zones")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let calc_payload = serde_json::json!({
        "country": "US",
        "subtotal": "80.00",
        "weight_kg": "2.5"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/shipping/calculate")
        .set_json(&calc_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let shipping_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(shipping_res["status"], "OK");
    assert!(!shipping_res["options"].as_array().unwrap().is_empty());

    // =========================================================================
    // 6. Commerce: Order State Transitions & RMA
    // =========================================================================
    let ord_payload = serde_json::json!({
        "current_state": "PendingPayment",
        "event": "confirm_payment"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/orders/transition")
        .set_json(&ord_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let ord_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ord_res["to_state"], "Processing");

    let rma_payload = serde_json::json!({
        "order_id": "ORD-2026-999",
        "customer_id": "cust_john",
        "items": [
            {
                "item_code": "LAPTOP-01",
                "qty": "1.0",
                "condition": "Defective",
                "reason": "Defective screen",
                "unit_refund_rate": "1200.00"
            }
        ],
        "reason": "Item arrived damaged"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/trade/rma")
        .set_json(&rma_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // =========================================================================
    // 7. CMS: Taxonomies, Media Library, SEO Engine
    // =========================================================================
    let req = test::TestRequest::get()
        .uri("/api/v2/cms/taxonomy")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let seo_payload = serde_json::json!({
        "title": "Enterprise Cloud Solutions",
        "description": "Next-generation distributed business OS in pure Rust.",
        "canonical_url": "https://rustnext.rs/cloud",
        "og_type": "website",
        "site_name": "RustNext"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/cms/seo/generate")
        .set_json(&seo_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let seo_html = String::from_utf8(body.to_vec()).unwrap();
    assert!(seo_html.contains("<title>Enterprise Cloud Solutions</title>"));
    assert!(seo_html.contains("og:title"));

    // =========================================================================
    // 8. Workflows & Time-Travel Version History
    // =========================================================================
    let wf_payload = serde_json::json!({
        "doctype": "Purchase Order",
        "doc_name": "PO-2026-001",
        "current_state": "Draft",
        "action": "submit_for_approval",
        "user_id": "usr_purchase",
        "user_roles": ["Purchase User"],
        "comment": "Submitting PO for review",
        "doc_data": {"grand_total": 5000.0}
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/workflow/evaluate")
        .set_json(&wf_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let wf_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(wf_res["next_state"], "Pending Approval");

    // Check version history
    let req = test::TestRequest::get()
        .uri("/api/v2/workflow/versions/Purchase%20Order/PO-2026-001")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let hist_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(hist_res["status"], "OK");
    assert_eq!(hist_res["count"], 1);

    // =========================================================================
    // 9. Universal Exporter & Dynamic Pivot Reporting
    // =========================================================================
    let export_payload = serde_json::json!({
        "format": "csv",
        "columns": [
            {"fieldname": "item_code", "header_label": "Item Code"},
            {"fieldname": "revenue", "header_label": "Revenue"}
        ],
        "rows": [
            {"item_code": "LAPTOP-01", "revenue": "12000.00"},
            {"item_code": "DESKTOP-02", "revenue": "8500.00"}
        ]
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/export/data")
        .set_json(&export_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let csv_out = String::from_utf8(body.to_vec()).unwrap();
    assert!(csv_out.contains("Item Code,Revenue"));
    assert!(csv_out.contains("LAPTOP-01,12000.00"));

    let pivot_payload = serde_json::json!({
        "row_field": "region",
        "column_field": "category",
        "value_field": "amount",
        "aggregation": "sum",
        "rows": [
            {"region": "North", "category": "Hardware", "amount": 5000.0},
            {"region": "North", "category": "Software", "amount": 3000.0},
            {"region": "South", "category": "Hardware", "amount": 7000.0}
        ]
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/reports/pivot")
        .set_json(&pivot_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let pivot_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pivot_res["status"], "OK");
    assert_eq!(pivot_res["pivot_table"]["grand_total"], 15000.0);

    // =========================================================================
    // 10. Notifications & Webhooks Signed Dispatch
    // =========================================================================
    let notif_payload = serde_json::json!({
        "recipient_id": "usr_worker",
        "title": "Safety Drill Scheduled",
        "message": "Mandatory safety training at 14:00.",
        "channel": "InApp",
        "priority": "High"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/notifications/dispatch")
        .set_json(&notif_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/api/v2/notifications/inbox/usr_worker")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let wh_payload = serde_json::json!({
        "target_url": "https://api.external.com/webhook",
        "secret": "super_secret_enterprise_key",
        "event": "doc.submitted",
        "doctype": "Purchase Order",
        "docname": "PO-001",
        "data": {"grand_total": 5000.0}
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/webhooks/test")
        .set_json(&wh_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let wh_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        wh_res["computed_hmac_sha256"]
            .as_str()
            .unwrap()
            .starts_with("sha256=")
    );

    // =========================================================================
    // 11. Security Hardening: Token Authentication & Refresh Token Rotation Flow
    // =========================================================================
    let login_payload = serde_json::json!({
        "usr": "usr_admin",
        "pwd": "admin"
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/method/login")
        .set_json(&login_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let login_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(login_res["message"], "Logged In");
    let access_token = login_res["access_token"].as_str().unwrap();
    let refresh_token = login_res["refresh_token"].as_str().unwrap();
    assert!(access_token.starts_with("v4.local."));
    assert!(refresh_token.starts_with("rft_usr_admin"));

    // Rotate Refresh Token
    let refresh_payload = serde_json::json!({
        "refresh_token": refresh_token
    });
    let req = test::TestRequest::post()
        .uri("/api/v2/auth/refresh")
        .set_json(&refresh_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let refresh_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let new_access = refresh_res["access_token"].as_str().unwrap();
    let new_refresh = refresh_res["refresh_token"].as_str().unwrap();
    assert!(new_access.starts_with("v4.local."));
    assert!(new_refresh.starts_with("rft_usr_admin"));
    assert_ne!(refresh_token, new_refresh); // Verify rotated token is distinct

    // Old Refresh Token must now be rejected (single-use rotation policy)
    let req = test::TestRequest::post()
        .uri("/api/v2/auth/refresh")
        .set_json(&refresh_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    // =========================================================================
    // 12. Enterprise Print Format & POS Thermal Receipt Generation
    // =========================================================================
    let invoice_print_payload = serde_json::json!({
        "document_title": "Tax Invoice",
        "document_number": "ACC-INV-2026-00099",
        "posting_date": "2026-09-30T12:00:00Z",
        "due_date": "2026-10-30",
        "company_name": "RustNext Enterprise Corp",
        "company_tax_id": "VAT-987654321",
        "company_address": "42 Innovation Park, Tehran, Iran",
        "customer_name": "Faez Barghasa",
        "customer_tax_id": "CUST-VAT-1122",
        "customer_address": "Tech District, Tehran",
        "currency": "USD",
        "items": [
            {
                "item_code": "RUST-CORE-SERVER",
                "description": "High-Performance Bare-Metal Rust Server",
                "qty": "2",
                "unit_price": "2500.00",
                "discount_pct": "10",
                "tax_rate_pct": "15",
                "line_total": "4500.00"
            }
        ],
        "net_total": "5000.00",
        "total_discount": "500.00",
        "total_tax": "675.00",
        "grand_total": "5175.00",
        "qr_data": "ZATCA_SIMULATED_TAG_LENGTH_VALUE_992288",
        "terms_and_conditions": "Net 30 days payment term. Zero tolerance for security vulnerabilities."
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/method/render_invoice")
        .set_json(&invoice_print_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/html; charset=utf-8"
    );
    let body = test::read_body(resp).await;
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Tax Invoice"));
    assert!(html.contains("ACC-INV-2026-00099"));
    assert!(html.contains("RustNext Enterprise Corp"));
    assert!(html.contains("RUST-CORE-SERVER"));
    assert!(html.contains("5175.00"));
    assert!(html.contains("Jalali:"));

    // POS Thermal Receipt
    let receipt_payload = serde_json::json!({
        "store_name": "RustNext Central Store",
        "terminal_id": "POS-01",
        "cashier_name": "Admin",
        "receipt_number": "RCP-998811",
        "timestamp": "2026-09-30 14:00",
        "items": [
            {
                "item_code": "Coffee-Dark",
                "description": "Artisan Dark Roast 1kg",
                "qty": "1",
                "unit_price": "30.00",
                "discount_pct": "0",
                "tax_rate_pct": "0",
                "line_total": "30.00"
            }
        ],
        "subtotal": "30.00",
        "discount": "0.00",
        "tax": "3.00",
        "total": "33.00",
        "payment_method": "Contactless NFC",
        "change_due": "0.00"
    });

    let req = test::TestRequest::post()
        .uri("/api/v2/method/render_receipt")
        .set_json(&receipt_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let receipt_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(receipt_text.contains("RustNext Central Store"));
    assert!(receipt_text.contains("Coffee-Dark"));
    assert!(receipt_text.contains("33.00"));
}
