use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Material Request line item to be procured.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequisitionItem {
    /// Item code.
    pub item_code: String,
    /// Required quantity.
    pub qty: Decimal,
    /// Default supplier (if configured).
    pub default_supplier: Option<String>,
    /// Required by date.
    pub schedule_date: NaiveDate,
}

/// Material Request document resulting from splitting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterialRequestDoc {
    /// Target supplier (None for unassigned items).
    pub supplier: Option<String>,
    /// Item lines allocated to this request.
    pub items: Vec<RequisitionItem>,
}

/// Splits a combined multi-item requisition into separate Material Requests grouped by supplier.
pub fn split_material_requests_by_supplier(
    requisition_items: Vec<RequisitionItem>,
) -> Vec<MaterialRequestDoc> {
    let mut groups: HashMap<Option<String>, Vec<RequisitionItem>> = HashMap::new();

    for item in requisition_items {
        groups
            .entry(item.default_supplier.clone())
            .or_default()
            .push(item);
    }

    groups
        .into_iter()
        .map(|(supplier, items)| MaterialRequestDoc { supplier, items })
        .collect()
}

/// Quotation line item supporting optional & alternative configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuotationItem {
    /// Item code.
    pub item_code: String,
    /// Quantity.
    pub qty: Decimal,
    /// Rate.
    pub rate: Decimal,
    /// Optional item flag (excluded from grand total if true).
    pub is_optional: bool,
    /// Alternative item flag referencing primary item code.
    pub is_alternative_to: Option<String>,
}

impl QuotationItem {
    /// Total amount for the line item.
    pub fn amount(&self) -> Decimal {
        self.qty * self.rate
    }
}

/// Calculates totals for a quotation, strictly excluding optional items.
pub fn calculate_quotation_totals(items: &[QuotationItem]) -> (Decimal, Decimal) {
    let mut total_quoted = Decimal::ZERO;
    let mut total_optional = Decimal::ZERO;

    for item in items {
        let amt = item.amount();
        if item.is_optional {
            total_optional += amt;
        } else {
            total_quoted += amt;
        }
    }

    (total_quoted, total_optional)
}

/// Order status for bulk update operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderWorkflowStatus {
    /// Open active order.
    Open,
    /// Completed / Fulfilled.
    Completed,
    /// Closed / Short-closed.
    Closed,
    /// Reopened from closed state.
    Reopened,
}

/// Bulk updater for order documents.
pub struct BulkOrderManager;

impl BulkOrderManager {
    /// Updates status of multiple orders in bulk.
    pub fn bulk_set_status(
        orders: &mut [(&mut String, &mut OrderWorkflowStatus)],
        new_status: OrderWorkflowStatus,
    ) {
        for (_id, status) in orders.iter_mut() {
            **status = new_status;
        }
    }
}

/// In-place item line modifier for submitted orders (avoiding `-1`, `-2` amendment cloning).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InPlaceItemUpdate {
    /// Line item index or identifier.
    pub item_code: String,
    /// New quantity (if updated).
    pub new_qty: Option<Decimal>,
    /// New rate (if updated).
    pub new_rate: Option<Decimal>,
    /// New promised delivery date (if updated).
    pub new_delivery_date: Option<NaiveDate>,
}

/// Item line on a submitted sales/purchase order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderItemLine {
    /// Item code.
    pub item_code: String,
    /// Current quantity.
    pub qty: Decimal,
    /// Current rate.
    pub rate: Decimal,
    /// Current delivery date.
    pub delivery_date: NaiveDate,
}

impl OrderItemLine {
    /// Applies in-place item updates directly to the line item.
    pub fn apply_update(&mut self, update: &InPlaceItemUpdate) {
        if let Some(qty) = update.new_qty {
            self.qty = qty;
        }
        if let Some(rate) = update.new_rate {
            self.rate = rate;
        }
        if let Some(date) = update.new_delivery_date {
            self.delivery_date = date;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_split_material_requests() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let items = vec![
            RequisitionItem {
                item_code: "STEEL-BOLT".into(),
                qty: dec!(100),
                default_supplier: Some("SUPP-FASTENERS".into()),
                schedule_date: date,
            },
            RequisitionItem {
                item_code: "STEEL-NUT".into(),
                qty: dec!(100),
                default_supplier: Some("SUPP-FASTENERS".into()),
                schedule_date: date,
            },
            RequisitionItem {
                item_code: "MICROCONTROLLER".into(),
                qty: dec!(10),
                default_supplier: Some("SUPP-CHIPS".into()),
                schedule_date: date,
            },
            RequisitionItem {
                item_code: "MISC-PACKING".into(),
                qty: dec!(5),
                default_supplier: None,
                schedule_date: date,
            },
        ];

        let splits = split_material_requests_by_supplier(items);
        assert_eq!(splits.len(), 3);
    }

    #[test]
    fn test_quotation_optional_items() {
        let items = vec![
            QuotationItem {
                item_code: "BASE-SERVER".into(),
                qty: dec!(1),
                rate: dec!(2000.0),
                is_optional: false,
                is_alternative_to: None,
            },
            QuotationItem {
                item_code: "OPTIONAL-EXT-WARRANTY".into(),
                qty: dec!(1),
                rate: dec!(300.0),
                is_optional: true,
                is_alternative_to: None,
            },
        ];

        let (total_quoted, total_opt) = calculate_quotation_totals(&items);
        assert_eq!(total_quoted, dec!(2000.0));
        assert_eq!(total_opt, dec!(300.0));
    }

    #[test]
    fn test_inplace_item_update() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let mut line = OrderItemLine {
            item_code: "ITEM-X".into(),
            qty: dec!(10),
            rate: dec!(50.0),
            delivery_date: date,
        };

        let new_date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let update = InPlaceItemUpdate {
            item_code: "ITEM-X".into(),
            new_qty: Some(dec!(12)),
            new_rate: Some(dec!(48.0)),
            new_delivery_date: Some(new_date),
        };

        line.apply_update(&update);
        assert_eq!(line.qty, dec!(12));
        assert_eq!(line.rate, dec!(48.0));
        assert_eq!(line.delivery_date, new_date);
    }
}
