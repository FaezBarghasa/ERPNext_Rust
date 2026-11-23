//! Integration tests verifying Phase 3 Cross-Document Fulfillment & Lifecycle State Machine.
//!
//! Validates:
//! 1. Sales Order lifecycle: Draft -> To Deliver and Bill.
//! 2. Delivery Note dispatch updating delivered_qty on linked lines and transitioning to Partly Delivered -> Fully Delivered.
//! 3. Sales Invoice submission updating billed_amt on linked lines and transitioning to Completed.
//! 4. Document cancellation reversing delivered/billed quantities and restoring previous fulfillment states.
//! 5. Document amendment engine suffix increment rules and draft generation.

use rust_decimal_macros::dec;
use rwl_framework::fulfillment::{DocumentAmendmentEngine, FulfillmentTracker};
use serde_json::{Value, json};

#[test]
fn test_cross_document_fulfillment_and_reversal_cycle() {
    // 1. Setup Sales Order document
    let mut so_doc = json!({
        "name": "SO-2026-00042",
        "doctype": "Sales Order",
        "customer": "Wayne Enterprises",
        "docstatus": 1,
        "items": [
            {
                "name": "SO-ITEM-001",
                "item_code": "TITANIUM-SHEET",
                "qty": "10",
                "rate": "150.00",
                "amount": "1500.00",
                "delivered_qty": "0",
                "billed_amt": "0"
            },
            {
                "name": "SO-ITEM-002",
                "item_code": "GRAPHITE-ROD",
                "qty": "20",
                "rate": "25.00",
                "amount": "500.00",
                "delivered_qty": "0",
                "billed_amt": "0"
            }
        ]
    });

    // Initial check: Should be "To Deliver and Bill"
    let initial_status = FulfillmentTracker::calculate_sales_order_fulfillment(&so_doc);
    assert_eq!(initial_status.per_delivered, dec!(0));
    assert_eq!(initial_status.per_billed, dec!(0));
    assert_eq!(initial_status.status, "To Deliver and Bill");
    assert!(!initial_status.is_completed);

    // 2. Dispatch Partial Delivery Note (5 Titanium Sheets)
    let dn_partial = json!({
        "name": "MAT-DN-2026-00001",
        "doctype": "Delivery Note",
        "sales_order": "SO-2026-00042",
        "items": [
            {
                "item_code": "TITANIUM-SHEET",
                "so_detail": "SO-ITEM-001",
                "qty": "5"
            }
        ]
    });

    FulfillmentTracker::apply_delivery_note(&mut so_doc, &dn_partial);
    assert_eq!(so_doc["items"][0]["delivered_qty"], "5");
    assert_eq!(so_doc["status"], "Partly Delivered");
    // 5 * 150 = 750 delivered out of (10*150 + 20*25 = 2000) => 750 / 2000 * 100 = 37.5%
    assert_eq!(so_doc["per_delivered"], "37.5");

    // 3. Dispatch Second Delivery Note (5 Titanium Sheets + 20 Graphite Rods) -> Fully Delivered
    let dn_remaining = json!({
        "name": "MAT-DN-2026-00002",
        "doctype": "Delivery Note",
        "sales_order": "SO-2026-00042",
        "items": [
            {
                "item_code": "TITANIUM-SHEET",
                "so_detail": "SO-ITEM-001",
                "qty": "5"
            },
            {
                "item_code": "GRAPHITE-ROD",
                "so_detail": "SO-ITEM-002",
                "qty": "20"
            }
        ]
    });

    FulfillmentTracker::apply_delivery_note(&mut so_doc, &dn_remaining);
    assert_eq!(so_doc["items"][0]["delivered_qty"], "10");
    assert_eq!(so_doc["items"][1]["delivered_qty"], "20");
    assert_eq!(so_doc["per_delivered"], "100");
    assert_eq!(so_doc["status"], "To Bill"); // 100% delivered, 0% billed

    // 4. Issue Full Sales Invoice -> Transitions to Completed
    let si_full = json!({
        "name": "ACC-SINV-2026-00001",
        "doctype": "Sales Invoice",
        "sales_order": "SO-2026-00042",
        "items": [
            {
                "item_code": "TITANIUM-SHEET",
                "so_detail": "SO-ITEM-001",
                "amount": "1500.00"
            },
            {
                "item_code": "GRAPHITE-ROD",
                "so_detail": "SO-ITEM-002",
                "amount": "500.00"
            }
        ]
    });

    FulfillmentTracker::apply_sales_invoice(&mut so_doc, &si_full);
    assert_eq!(so_doc["items"][0]["billed_amt"], "1500.00");
    assert_eq!(so_doc["items"][1]["billed_amt"], "500.00");
    assert_eq!(so_doc["per_billed"], "100");
    assert_eq!(so_doc["status"], "Completed");

    // 5. Test Cancellation Reversal: Cancel the second delivery note
    FulfillmentTracker::reverse_delivery_note(&mut so_doc, &dn_remaining);
    assert_eq!(so_doc["items"][0]["delivered_qty"], "5");
    assert_eq!(so_doc["items"][1]["delivered_qty"], "0");
    assert_eq!(so_doc["status"], "To Deliver"); // 100% billed, 37.5% delivered

    // Cancel the Sales Invoice
    FulfillmentTracker::reverse_sales_invoice(&mut so_doc, &si_full);
    assert_eq!(so_doc["items"][0]["billed_amt"], "0");
    assert_eq!(so_doc["items"][1]["billed_amt"], "0");
    assert_eq!(so_doc["status"], "Partly Delivered"); // 37.5% delivered, 0% billed
}

#[test]
fn test_document_amendment_engine() {
    assert_eq!(
        DocumentAmendmentEngine::compute_amended_name("SO-2026-00042"),
        "SO-2026-00042-1"
    );
    assert_eq!(
        DocumentAmendmentEngine::compute_amended_name("SO-2026-00042-1"),
        "SO-2026-00042-2"
    );
    assert_eq!(
        DocumentAmendmentEngine::compute_amended_name("ACC-SINV-2026-00001-9"),
        "ACC-SINV-2026-00001-10"
    );

    // Cancelled document
    let cancelled_doc = json!({
        "name": "ACC-SINV-2026-00001",
        "doctype": "Sales Invoice",
        "docstatus": 2, // Cancelled
        "customer": "Wayne Enterprises",
        "grand_total": "2000.00"
    });

    let obj = cancelled_doc.as_object().unwrap().clone();
    let amended = DocumentAmendmentEngine::prepare_amended_document(obj).unwrap();

    assert_eq!(amended["name"], "ACC-SINV-2026-00001-1");
    assert_eq!(amended["amended_from"], "ACC-SINV-2026-00001");
    assert_eq!(amended["docstatus"], 0); // Draft
    assert_eq!(amended["workflow_state"], "Draft");
    assert_eq!(amended["customer"], "Wayne Enterprises");
}
