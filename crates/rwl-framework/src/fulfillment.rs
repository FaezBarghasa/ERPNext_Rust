//! Cross-Document Fulfillment & Lifecycle State Machine (`rwl-framework::fulfillment`).
//!
//! Implements Phase 3 requirements:
//! - Tracks line-item linkages (`sales_order_item`, `delivery_note_item`, `purchase_order_item`, `purchase_receipt_item`).
//! - Automated delivery fulfillment calculations:
//!   $$\text{per\_delivered} = \frac{\sum \text{delivered\_qty} \times \text{rate}}{\sum \text{qty} \times \text{rate}} \times 100$$
//! - Automated billing fulfillment calculations:
//!   $$\text{per\_billed} = \frac{\sum \text{billed\_amt}}{\sum \text{amount}} \times 100$$
//! - Automated document status transitions:
//!   * `Draft` -> `To Deliver and Bill` -> `Partly Delivered` -> `Fully Delivered` -> `Completed`
//! - Reversible fulfillment adjustments upon document cancellation.
//! - Deterministic document amendment engine handling name suffix increments.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Cross-document line linkage descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineItemLinkage {
    pub item_code: String,
    pub qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub delivered_qty: Decimal,
    pub billed_amt: Decimal,
    pub sales_order: Option<String>,
    pub sales_order_item: Option<String>,
    pub purchase_order: Option<String>,
    pub purchase_order_item: Option<String>,
}

/// Overall Document Fulfillment Status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentFulfillmentStatus {
    pub per_delivered: Decimal,
    pub per_billed: Decimal,
    pub status: String,
    pub is_completed: bool,
}

/// Cross-Document Fulfillment Tracker.
pub struct FulfillmentTracker;

impl FulfillmentTracker {
    /// Helper to safely extract Decimal.
    pub fn parse_decimal(v: Option<&Value>) -> Decimal {
        match v {
            Some(Value::String(s)) => s.parse::<Decimal>().unwrap_or(Decimal::ZERO),
            Some(Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    Decimal::from(i)
                } else if let Some(f) = n.as_f64() {
                    Decimal::try_from(f).unwrap_or(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                }
            }
            _ => Decimal::ZERO,
        }
    }

    /// Computes overall fulfillment percentages and canonical status for a Sales Order document.
    pub fn calculate_sales_order_fulfillment(so_doc: &Value) -> DocumentFulfillmentStatus {
        let items = so_doc.get("items").and_then(|v| v.as_array());

        let mut total_ordered_val = Decimal::ZERO;
        let mut total_delivered_val = Decimal::ZERO;
        let mut total_ordered_amt = Decimal::ZERO;
        let mut total_billed_amt = Decimal::ZERO;

        if let Some(item_list) = items {
            for itm in item_list {
                let qty = Self::parse_decimal(itm.get("qty"));
                let rate = Self::parse_decimal(itm.get("rate"));
                let amt = if itm.get("amount").is_some() {
                    Self::parse_decimal(itm.get("amount"))
                } else {
                    qty * rate
                };
                let delivered_qty = Self::parse_decimal(itm.get("delivered_qty"));
                let billed_amt = Self::parse_decimal(itm.get("billed_amt"));

                total_ordered_val += qty * rate;
                total_delivered_val += delivered_qty * rate;
                total_ordered_amt += amt;
                total_billed_amt += billed_amt;
            }
        }

        let per_delivered = if total_ordered_val > Decimal::ZERO {
            (((total_delivered_val / total_ordered_val) * Decimal::from(100)).min(Decimal::from(100))).normalize()
        } else {
            Decimal::ZERO
        };

        let per_billed = if total_ordered_amt > Decimal::ZERO {
            (((total_billed_amt / total_ordered_amt) * Decimal::from(100)).min(Decimal::from(100))).normalize()
        } else {
            Decimal::ZERO
        };

        let is_completed = per_delivered >= Decimal::from(100) && per_billed >= Decimal::from(100);

        let status = if is_completed {
            "Completed".to_string()
        } else if per_delivered >= Decimal::from(100) {
            "To Bill".to_string()
        } else if per_billed >= Decimal::from(100) {
            "To Deliver".to_string()
        } else if per_delivered > Decimal::ZERO || per_billed > Decimal::ZERO {
            "Partly Delivered".to_string()
        } else {
            "To Deliver and Bill".to_string()
        };

        DocumentFulfillmentStatus {
            per_delivered,
            per_billed,
            status,
            is_completed,
        }
    }

    /// Applies Delivery Note dispatch against parent Sales Order lines.
    ///
    /// Increments `delivered_qty` on matching items and recalculates Sales Order status.
    pub fn apply_delivery_note(so_doc: &mut Value, dn_doc: &Value) {
        let Some(dn_items) = dn_doc.get("items").and_then(|v| v.as_array()) else {
            return;
        };

        if let Some(so_items) = so_doc.get_mut("items").and_then(|v| v.as_array_mut()) {
            for dn_item in dn_items {
                let dn_qty = Self::parse_decimal(dn_item.get("qty"));
                let so_item_link = dn_item.get("so_detail").or_else(|| dn_item.get("sales_order_item")).and_then(|v| v.as_str());
                let dn_item_code = dn_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                for so_item in so_items.iter_mut() {
                    let so_item_name = so_item.get("name").and_then(|v| v.as_str());
                    let so_code = so_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                    let matches = match so_item_link {
                        Some(link) => so_item_name == Some(link),
                        None => so_code == dn_item_code,
                    };

                    if matches {
                        let cur_delivered = Self::parse_decimal(so_item.get("delivered_qty"));
                        let new_delivered = cur_delivered + dn_qty;
                        if let Some(obj) = so_item.as_object_mut() {
                            obj.insert("delivered_qty".into(), Value::String(new_delivered.to_string()));
                        }
                        break;
                    }
                }
            }
        }

        // Recompute parent statuses
        let fulfillment = Self::calculate_sales_order_fulfillment(so_doc);
        if let Some(obj) = so_doc.as_object_mut() {
            obj.insert("per_delivered".into(), Value::String(fulfillment.per_delivered.to_string()));
            obj.insert("per_billed".into(), Value::String(fulfillment.per_billed.to_string()));
            obj.insert("status".into(), Value::String(fulfillment.status));
        }
    }

    /// Reverses Delivery Note dispatch against parent Sales Order lines upon cancellation.
    pub fn reverse_delivery_note(so_doc: &mut Value, dn_doc: &Value) {
        let Some(dn_items) = dn_doc.get("items").and_then(|v| v.as_array()) else {
            return;
        };

        if let Some(so_items) = so_doc.get_mut("items").and_then(|v| v.as_array_mut()) {
            for dn_item in dn_items {
                let dn_qty = Self::parse_decimal(dn_item.get("qty"));
                let so_item_link = dn_item.get("so_detail").or_else(|| dn_item.get("sales_order_item")).and_then(|v| v.as_str());
                let dn_item_code = dn_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                for so_item in so_items.iter_mut() {
                    let so_item_name = so_item.get("name").and_then(|v| v.as_str());
                    let so_code = so_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                    let matches = match so_item_link {
                        Some(link) => so_item_name == Some(link),
                        None => so_code == dn_item_code,
                    };

                    if matches {
                        let cur_delivered = Self::parse_decimal(so_item.get("delivered_qty"));
                        let new_delivered = (cur_delivered - dn_qty).max(Decimal::ZERO).normalize();
                        if let Some(obj) = so_item.as_object_mut() {
                            obj.insert("delivered_qty".into(), Value::String(new_delivered.to_string()));
                        }
                        break;
                    }
                }
            }
        }

        let fulfillment = Self::calculate_sales_order_fulfillment(so_doc);
        if let Some(obj) = so_doc.as_object_mut() {
            obj.insert("per_delivered".into(), Value::String(fulfillment.per_delivered.to_string()));
            obj.insert("per_billed".into(), Value::String(fulfillment.per_billed.to_string()));
            obj.insert("status".into(), Value::String(fulfillment.status));
        }
    }

    /// Applies Sales Invoice billing against parent Sales Order lines.
    pub fn apply_sales_invoice(so_doc: &mut Value, si_doc: &Value) {
        let Some(si_items) = si_doc.get("items").and_then(|v| v.as_array()) else {
            return;
        };

        if let Some(so_items) = so_doc.get_mut("items").and_then(|v| v.as_array_mut()) {
            for si_item in si_items {
                let si_amt = if si_item.get("amount").is_some() {
                    Self::parse_decimal(si_item.get("amount"))
                } else {
                    Self::parse_decimal(si_item.get("qty")) * Self::parse_decimal(si_item.get("rate"))
                };
                let so_item_link = si_item.get("so_detail").or_else(|| si_item.get("sales_order_item")).and_then(|v| v.as_str());
                let si_item_code = si_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                for so_item in so_items.iter_mut() {
                    let so_item_name = so_item.get("name").and_then(|v| v.as_str());
                    let so_code = so_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                    let matches = match so_item_link {
                        Some(link) => so_item_name == Some(link),
                        None => so_code == si_item_code,
                    };

                    if matches {
                        let cur_billed = Self::parse_decimal(so_item.get("billed_amt"));
                        let new_billed = cur_billed + si_amt;
                        if let Some(obj) = so_item.as_object_mut() {
                            obj.insert("billed_amt".into(), Value::String(new_billed.to_string()));
                        }
                        break;
                    }
                }
            }
        }

        let fulfillment = Self::calculate_sales_order_fulfillment(so_doc);
        if let Some(obj) = so_doc.as_object_mut() {
            obj.insert("per_delivered".into(), Value::String(fulfillment.per_delivered.to_string()));
            obj.insert("per_billed".into(), Value::String(fulfillment.per_billed.to_string()));
            obj.insert("status".into(), Value::String(fulfillment.status));
        }
    }

    /// Reverses Sales Invoice billing upon cancellation.
    pub fn reverse_sales_invoice(so_doc: &mut Value, si_doc: &Value) {
        let Some(si_items) = si_doc.get("items").and_then(|v| v.as_array()) else {
            return;
        };

        if let Some(so_items) = so_doc.get_mut("items").and_then(|v| v.as_array_mut()) {
            for si_item in si_items {
                let si_amt = if si_item.get("amount").is_some() {
                    Self::parse_decimal(si_item.get("amount"))
                } else {
                    Self::parse_decimal(si_item.get("qty")) * Self::parse_decimal(si_item.get("rate"))
                };
                let so_item_link = si_item.get("so_detail").or_else(|| si_item.get("sales_order_item")).and_then(|v| v.as_str());
                let si_item_code = si_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                for so_item in so_items.iter_mut() {
                    let so_item_name = so_item.get("name").and_then(|v| v.as_str());
                    let so_code = so_item.get("item_code").and_then(|v| v.as_str()).unwrap_or("");

                    let matches = match so_item_link {
                        Some(link) => so_item_name == Some(link),
                        None => so_code == si_item_code,
                    };

                    if matches {
                        let cur_billed = Self::parse_decimal(so_item.get("billed_amt"));
                        let new_billed = (cur_billed - si_amt).max(Decimal::ZERO).normalize();
                        if let Some(obj) = so_item.as_object_mut() {
                            obj.insert("billed_amt".into(), Value::String(new_billed.to_string()));
                        }
                        break;
                    }
                }
            }
        }

        let fulfillment = Self::calculate_sales_order_fulfillment(so_doc);
        if let Some(obj) = so_doc.as_object_mut() {
            obj.insert("per_delivered".into(), Value::String(fulfillment.per_delivered.to_string()));
            obj.insert("per_billed".into(), Value::String(fulfillment.per_billed.to_string()));
            obj.insert("status".into(), Value::String(fulfillment.status));
        }
    }
}

/// Formal Document Amendment Engine.
pub struct DocumentAmendmentEngine;

impl DocumentAmendmentEngine {
    /// Computes the amended document name adhering to Frappe/ERPNext suffix increments:
    /// - `SO-2026-0001` -> `SO-2026-0001-1`
    /// - `SO-2026-0001-1` -> `SO-2026-0001-2`
    pub fn compute_amended_name(original_name: &str) -> String {
        if let Some((base, suffix)) = original_name.rsplit_once('-')
            && let Ok(num) = suffix.parse::<u32>()
            && suffix.len() <= 3
            && base.contains('-')
        {
            return format!("{}-{}", base, num + 1);
        }
        format!("{}-1", original_name)
    }

    /// Clones and prepares an amended draft document from a cancelled original.
    pub fn prepare_amended_document(mut orig_doc: Map<String, Value>) -> Result<Map<String, Value>, String> {
        let orig_name = orig_doc
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Original document has no 'name' field".to_string())?
            .to_string();

        let docstatus = orig_doc
            .get("docstatus")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if docstatus != 2 {
            return Err(format!("Document {orig_name} must be cancelled (docstatus = 2) to be amended, found {docstatus}"));
        }

        let new_name = Self::compute_amended_name(&orig_name);

        orig_doc.insert("name".into(), Value::String(new_name));
        orig_doc.insert("amended_from".into(), Value::String(orig_name));
        orig_doc.insert("docstatus".into(), Value::Number(0.into())); // Reset to Draft
        orig_doc.insert("workflow_state".into(), Value::String("Draft".into()));

        // Strip previous posting/audit stamps
        let now = chrono::Utc::now().to_rfc3339();
        orig_doc.insert("creation".into(), Value::String(now.clone()));
        orig_doc.insert("modified".into(), Value::String(now));

        Ok(orig_doc)
    }
}
