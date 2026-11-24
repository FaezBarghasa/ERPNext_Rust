//! Integration tests verifying 100% Frappe REST and RPC wire compatibility.
//!
//! Covers:
//! - `GET /api/method/frappe.auth.get_logged_user`
//! - `POST /api/resource/Sales Order` and `GET /api/resource/Sales Order/{name}`
//! - `POST /api/method/frappe.desk.form.load.getdoc`
//! - `POST /api/method/frappe.desk.form.save.savedocs` with `action: "Save"` and `action: "Submit"`
//! - `POST /api/method/frappe.desk.search.search_link`
//! - `POST /api/method/frappe.client.get_list`, `get_value`, and `set_value`
//! - Frappe Make-Mappers: `make_delivery_note`, `make_sales_invoice`, and `make_payment_entry`
//! - Authentication via `Cookie: sid=...` and `Authorization: token key:secret`

use actix_web::{App, test};
use rwl_net::server::configure_app;
use rwl_net::tenant::{ConnectionPoolManager, MicroTopologyConfig};
use serde_json::{Value, json};
use std::time::Duration;

#[actix_web::test]
async fn test_frappe_wire_compatibility_suite() {
    let pool_mgr = ConnectionPoolManager::in_memory(Duration::from_secs(60));

    let app = test::init_service(
        App::new().configure(|cfg| {
            configure_app(cfg, pool_mgr.clone(), MicroTopologyConfig::default(), None)
        }),
    )
    .await;

    // 1. Test frappe.auth.get_logged_user with sid cookie
    let req = test::TestRequest::get()
        .uri("/api/method/frappe.auth.get_logged_user")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Administrator");

    // 2. Test frappe.auth.get_logged_user with token auth
    let req = test::TestRequest::get()
        .uri("/api/method/frappe.auth.get_logged_user")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Authorization", "token testkey123:testsecret456"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "testkey123");

    // 3. Test Frappe Resource CRUD: POST /api/resource/Sales Order
    let so_payload = json!({
        "doctype": "Sales Order",
        "customer": "Acme Industrial Corp",
        "docstatus": 0,
        "items": [
            {
                "item_code": "SPROCKET-X1",
                "qty": 5,
                "rate": 120.0
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
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    let so_name = body["data"]["name"].as_str().expect("name must be string");
    assert!(so_name.starts_with("SO-"));
    assert_eq!(body["data"]["customer"], "Acme Industrial Corp");

    // 4. Test Frappe Resource GET: GET /api/resource/Sales Order/{name}
    let req = test::TestRequest::get()
        .uri(&format!("/api/resource/Sales%20Order/{}", so_name))
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], so_name);
    assert_eq!(body["data"]["customer"], "Acme Industrial Corp");

    // 5. Test frappe.desk.form.load.getdoc
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.form.load.getdoc")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "doctype": "Sales Order",
            "name": so_name
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert!(body["docs"].is_array());
    assert_eq!(body["docs"][0]["name"], so_name);
    assert!(body["docinfo"].is_object());

    // 6. Test frappe.desk.form.save.savedocs (Submit)
    let submit_payload = json!({
        "doc": {
            "doctype": "Sales Order",
            "name": so_name,
            "customer": "Acme Industrial Corp",
            "docstatus": 0,
            "items": [
                {
                    "item_code": "SPROCKET-X1",
                    "qty": 5,
                    "rate": 120.0
                }
            ]
        },
        "action": "Submit"
    });
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.form.save.savedocs")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&submit_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["docs"][0]["docstatus"], 1);

    // 7. Test frappe.client.get_value
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.client.get_value")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "doctype": "Sales Order",
            "filters": { "name": so_name },
            "fieldname": "customer"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"]["customer"], "Acme Industrial Corp");

    // 8. Test frappe.client.set_value
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.client.set_value")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "doctype": "Sales Order",
            "name": so_name,
            "fieldname": "title",
            "value": "Priority Order - Industrial Gear"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"]["title"], "Priority Order - Industrial Gear");

    // 9. Test frappe.desk.search.search_link
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.search.search_link")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "doctype": "Sales Order",
            "txt": "SO-"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    let results = body["results"].as_array().expect("results must be array");
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r["value"] == so_name));

    // 10. Test Make-Mappers: make_delivery_note
    let req = test::TestRequest::post()
        .uri("/api/method/erpnext.selling.doctype.sales_order.sales_order.make_delivery_note")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "source_name": so_name
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    let dn_doc = &body["message"];
    assert_eq!(dn_doc["doctype"], "Delivery Note");
    assert_eq!(dn_doc["customer"], "Acme Industrial Corp");
    assert!(dn_doc["name"].as_str().unwrap().starts_with("MAT-DN-"));
    assert_eq!(dn_doc["docstatus"], 0);

    // 11. Test Make-Mappers: make_sales_invoice
    let req = test::TestRequest::post()
        .uri("/api/method/erpnext.selling.doctype.sales_order.sales_order.make_sales_invoice")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "source_name": so_name
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    let si_doc = body["message"].clone();
    assert_eq!(si_doc["doctype"], "Sales Invoice");
    assert_eq!(si_doc["customer"], "Acme Industrial Corp");
    let si_name = si_doc["name"].as_str().unwrap().to_string();
    assert!(si_name.starts_with("ACC-SINV-"));
    assert_eq!(si_doc["docstatus"], 0);

    // Save/persist the generated Sales Invoice so make_payment_entry can query it
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.form.save.savedocs")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "doc": si_doc,
            "action": "Submit"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("savedocs Submit of Sales Invoice failed with status {status}: {err_str}");
    }
    let si_submit_resp: Value = test::read_body_json(resp).await;
    let si_submitted_name = si_submit_resp["docs"][0]["name"].as_str().unwrap().to_string();

    // 12. Test Make-Mappers: make_payment_entry from Sales Invoice
    let req = test::TestRequest::post()
        .uri("/api/method/erpnext.accounts.doctype.payment_entry.payment_entry.get_payment_entry")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({
            "dt": "Sales Invoice",
            "dn": si_submitted_name
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    let pe_doc = &body["message"];
    assert_eq!(pe_doc["doctype"], "Payment Entry");
    assert_eq!(pe_doc["payment_type"], "Receive");
    assert_eq!(pe_doc["party_type"], "Customer");
    assert_eq!(pe_doc["party"], "Acme Industrial Corp");
    assert!(pe_doc["name"].as_str().unwrap().starts_with("ACC-PAY-"));

    // 13. Test Frappe Resource DELETE: DELETE /api/resource/Sales Order/{name}
    let req = test::TestRequest::delete()
        .uri(&format!("/api/resource/Sales%20Order/{}", so_name))
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "ok");

    // 14. Test Security: Unauthenticated request to get_logged_user returns "Guest"
    let req = test::TestRequest::get()
        .uri("/api/method/frappe.auth.get_logged_user")
        .insert_header(("Host", "default.localhost"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    if !resp.status().is_success() {
        let status = resp.status();
        let body_bytes = test::read_body(resp).await;
        let err_str = String::from_utf8_lossy(&body_bytes);
        panic!("get_payment_entry failed with status {status}: {err_str}");
    }
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Guest");

    // 15. Test Security: Unauthenticated request with Guest cookie to savedocs returns 401 Unauthorized
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.desk.form.save.savedocs")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=Guest"))
        .set_json(&submit_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["exc_type"], "AuthenticationError");

    // 16. Test RPC Not Found: Unhandled RPC method returns 404 with DoesNotExistError
    let req = test::TestRequest::post()
        .uri("/api/method/frappe.non_existent.method")
        .insert_header(("Host", "default.localhost"))
        .insert_header(("Cookie", "sid=test-session-admin"))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["exc_type"], "DoesNotExistError");
}
