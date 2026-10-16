//! Commercial Flow Document Mapping Engine (`erp-trade::mapped_doc`).
//!
//! Replicates upstream Frappe `frappe.model.mapper.make_mapped_doc` pipeline
//! for standard Selling & Buying lifecycles:
//! - Quotation -> Sales Order -> Delivery Note -> Sales Invoice
//! - Material Request -> Request for Quotation -> Supplier Quotation -> Purchase Order -> Purchase Receipt -> Purchase Invoice
//! - Tracks pending / unfulfilled quantities and fulfills downstream child lines.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Mapping errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MappingError {
    #[error("Source document '{0}' has no pending unfulfilled items to map")]
    NoPendingItems(CompactString),
    #[error(
        "Quantity {requested} requested for item '{item_code}' exceeds pending balance {pending}"
    )]
    OverfulfilledQuantity {
        item_code: CompactString,
        requested: Decimal,
        pending: Decimal,
    },
}

/// Generic commercial line item in Orders, Receipts, and Invoices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommercialLineItem {
    pub item_code: CompactString,
    pub item_name: CompactString,
    pub uom: CompactString,
    pub conversion_factor: Decimal, // e.g. 1 Box = 12 Nos
    pub ordered_qty: Decimal,
    pub fulfilled_qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
    pub warehouse: Option<CompactString>,
}

impl CommercialLineItem {
    /// Returns the remaining unfulfilled balance on this line item.
    #[must_use]
    pub fn pending_qty(&self) -> Decimal {
        if self.ordered_qty > self.fulfilled_qty {
            self.ordered_qty - self.fulfilled_qty
        } else {
            Decimal::ZERO
        }
    }
}

/// Commercial header document representing Quotations, Orders, Shipments, Invoices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommercialDoc {
    pub name: CompactString,
    pub doctype: CompactString,
    pub party_type: CompactString, // "Customer" or "Supplier"
    pub party: CompactString,
    pub company: CompactString,
    pub currency: CompactString,
    pub items: Vec<CommercialLineItem>,
    pub net_total: Decimal,
}

/// Document Mapping Engine.
pub struct MappedDocEngine;

impl MappedDocEngine {
    /// Maps a source document to a target downstream document, transferring unfulfilled items.
    ///
    /// For instance:
    /// - `Sales Order` -> `Delivery Note`
    /// - `Purchase Order` -> `Purchase Receipt`
    pub fn make_mapped_doc(
        source: &mut CommercialDoc,
        target_doctype: &str,
        target_name: &str,
    ) -> Result<CommercialDoc, MappingError> {
        let mut target_items = Vec::new();
        let mut target_net_total = Decimal::ZERO;

        for item in &mut source.items {
            let pending = item.pending_qty();
            if pending > Decimal::ZERO {
                let amount = pending * item.rate;
                target_net_total += amount;

                target_items.push(CommercialLineItem {
                    item_code: item.item_code.clone(),
                    item_name: item.item_name.clone(),
                    uom: item.uom.clone(),
                    conversion_factor: item.conversion_factor,
                    ordered_qty: pending,
                    fulfilled_qty: Decimal::ZERO,
                    rate: item.rate,
                    amount,
                    warehouse: item.warehouse.clone(),
                });

                // Mark source fulfilled
                item.fulfilled_qty += pending;
            }
        }

        if target_items.is_empty() {
            return Err(MappingError::NoPendingItems(source.name.clone()));
        }

        Ok(CommercialDoc {
            name: CompactString::new(target_name),
            doctype: CompactString::new(target_doctype),
            party_type: source.party_type.clone(),
            party: source.party.clone(),
            company: source.company.clone(),
            currency: source.currency.clone(),
            items: target_items,
            net_total: target_net_total,
        })
    }

    /// Partially fulfills a specific line item in the source document.
    pub fn fulfill_item_qty(
        source: &mut CommercialDoc,
        item_code: &str,
        dispatch_qty: Decimal,
    ) -> Result<(), MappingError> {
        for item in &mut source.items {
            if item.item_code == item_code {
                let pending = item.pending_qty();
                if dispatch_qty > pending {
                    return Err(MappingError::OverfulfilledQuantity {
                        item_code: item.item_code.clone(),
                        requested: dispatch_qty,
                        pending,
                    });
                }
                item.fulfilled_qty += dispatch_qty;
                return Ok(());
            }
        }
        Err(MappingError::NoPendingItems(source.name.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_to_delivery_mapping_flow() {
        let mut so = CommercialDoc {
            name: "SAL-ORD-001".into(),
            doctype: "Sales Order".into(),
            party_type: "Customer".into(),
            party: "CUST-001".into(),
            company: "Solar Tech".into(),
            currency: "USD".into(),
            items: vec![
                CommercialLineItem {
                    item_code: "PANEL-400W".into(),
                    item_name: "400W Solar Panel".into(),
                    uom: "Nos".into(),
                    conversion_factor: Decimal::ONE,
                    ordered_qty: dec!(10.0),
                    fulfilled_qty: Decimal::ZERO,
                    rate: dec!(150.0),
                    amount: dec!(1500.0),
                    warehouse: Some("Finished Goods - ST".into()),
                },
                CommercialLineItem {
                    item_code: "INVERTER-5KW".into(),
                    item_name: "5KW Hybrid Inverter".into(),
                    uom: "Nos".into(),
                    conversion_factor: Decimal::ONE,
                    ordered_qty: dec!(2.0),
                    fulfilled_qty: Decimal::ZERO,
                    rate: dec!(800.0),
                    amount: dec!(1600.0),
                    warehouse: Some("Finished Goods - ST".into()),
                },
            ],
            net_total: dec!(3100.0),
        };

        // 1. Partial dispatch: 5 Panels
        MappedDocEngine::fulfill_item_qty(&mut so, "PANEL-400W", dec!(5.0)).unwrap();
        assert_eq!(so.items[0].pending_qty(), dec!(5.0));

        // 2. Map remaining to Delivery Note
        let dn = MappedDocEngine::make_mapped_doc(&mut so, "Delivery Note", "MAT-DN-001")
            .expect("Mapping to Delivery Note must succeed");

        assert_eq!(dn.doctype, "Delivery Note");
        assert_eq!(dn.name, "MAT-DN-001");
        assert_eq!(dn.items.len(), 2);
        // Panels in DN: 5 @ 150 = 750
        assert_eq!(dn.items[0].ordered_qty, dec!(5.0));
        assert_eq!(dn.items[0].amount, dec!(750.0));
        // Inverters in DN: 2 @ 800 = 1600
        assert_eq!(dn.items[1].ordered_qty, dec!(2.0));
        assert_eq!(dn.items[1].amount, dec!(1600.0));
        assert_eq!(dn.net_total, dec!(2350.0));

        // 3. SO should now be fully fulfilled
        assert_eq!(so.items[0].pending_qty(), Decimal::ZERO);
        assert_eq!(so.items[1].pending_qty(), Decimal::ZERO);

        // 4. Attempting to map fully fulfilled SO must fail
        assert_eq!(
            MappedDocEngine::make_mapped_doc(&mut so, "Delivery Note", "MAT-DN-002"),
            Err(MappingError::NoPendingItems("SAL-ORD-001".into()))
        );
    }
}
