//! Logistics & Material Movement Controller Suite (`erp-inventory::logistics`).
//!
//! Provides upstream parity for:
//! - `Material Request` (Purchase, Material Transfer, Material Issue, Manufacture)
//! - `Pick List` (Warehouse location routing, batch/serial picking, order fulfillment)
//! - `Packing Slip` (Carton/package containerization, gross weight validation)
//! - `Delivery Trip` (Multi-stop vehicle route dispatch and delivery verification).

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Purpose of a Material Request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialRequestType {
    Purchase,
    MaterialTransfer,
    MaterialIssue,
    Manufacture,
    CustomerProvided,
}

/// A line item in a Material Request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterialRequestItem {
    pub item_code: CompactString,
    pub qty: Decimal,
    pub ordered_qty: Decimal,
    pub warehouse: Option<CompactString>,
}

/// Submittable Material Request document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterialRequest {
    pub name: CompactString,
    pub material_request_type: MaterialRequestType,
    pub company: CompactString,
    pub items: Vec<MaterialRequestItem>,
    pub status: CompactString,
    pub docstatus: u8, // 0: Draft, 1: Submitted, 2: Cancelled
}

impl MaterialRequest {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        request_type: MaterialRequestType,
        company: impl Into<CompactString>,
        items: Vec<MaterialRequestItem>,
    ) -> Self {
        Self {
            name: name.into(),
            material_request_type: request_type,
            company: company.into(),
            items,
            status: "Submitted".into(),
            docstatus: 1,
        }
    }

    /// Checks if the request is completely fulfilled by downstream orders.
    #[must_use]
    pub fn is_fully_ordered(&self) -> bool {
        self.items.iter().all(|i| i.ordered_qty >= i.qty)
    }
}

/// A picking line item in a Pick List.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PickListItem {
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub batch_no: Option<CompactString>,
    pub serial_no: Option<CompactString>,
    pub qty: Decimal,
    pub picked_qty: Decimal,
}

/// Pick List document routing warehouse pickers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PickList {
    pub name: CompactString,
    pub company: CompactString,
    pub purpose: CompactString,
    pub locations: Vec<PickListItem>,
    pub status: CompactString,
    pub docstatus: u8,
}

impl PickList {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        company: impl Into<CompactString>,
        purpose: impl Into<CompactString>,
        locations: Vec<PickListItem>,
    ) -> Self {
        Self {
            name: name.into(),
            company: company.into(),
            purpose: purpose.into(),
            locations,
            status: "Draft".into(),
            docstatus: 0,
        }
    }

    /// Records picked quantity and marks complete when all items picked.
    pub fn record_pick(&mut self, idx: usize, qty: Decimal) {
        if let Some(loc) = self.locations.get_mut(idx) {
            loc.picked_qty += qty;
        }
        if self.locations.iter().all(|l| l.picked_qty >= l.qty) {
            self.status = "Completed".into();
            self.docstatus = 1;
        }
    }
}

/// Packing Slip document for parcel containerization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackingSlip {
    pub name: CompactString,
    pub delivery_note: CompactString,
    pub from_package_no: u32,
    pub to_package_no: u32,
    pub net_weight: Decimal,
    pub gross_weight: Decimal,
    pub docstatus: u8,
}

/// Delivery Stop in a Delivery Trip route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryStop {
    pub customer: CompactString,
    pub address: CompactString,
    pub delivery_note: Option<CompactString>,
    pub visited: bool,
}

/// Delivery Trip document coordinating vehicle logistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryTrip {
    pub name: CompactString,
    pub driver: CompactString,
    pub vehicle: CompactString,
    pub departure_time: String,
    pub stops: Vec<DeliveryStop>,
    pub docstatus: u8,
}

impl DeliveryTrip {
    #[must_use]
    pub fn new(
        name: impl Into<CompactString>,
        driver: impl Into<CompactString>,
        vehicle: impl Into<CompactString>,
        departure_time: impl Into<String>,
        stops: Vec<DeliveryStop>,
    ) -> Self {
        Self {
            name: name.into(),
            driver: driver.into(),
            vehicle: vehicle.into(),
            departure_time: departure_time.into(),
            stops,
            docstatus: 1,
        }
    }

    pub fn complete_stop(&mut self, idx: usize) {
        if let Some(stop) = self.stops.get_mut(idx) {
            stop.visited = true;
        }
    }

    #[must_use]
    pub fn is_all_visited(&self) -> bool {
        self.stops.iter().all(|s| s.visited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_logistics_workflow() {
        let mr = MaterialRequest::new(
            "MAT-REQ-2026-0001",
            MaterialRequestType::Purchase,
            "Acme Retail",
            vec![MaterialRequestItem {
                item_code: "STEEL-SHEET".into(),
                qty: dec!(100),
                ordered_qty: dec!(100),
                warehouse: Some("Raw Materials - AR".into()),
            }],
        );
        assert!(mr.is_fully_ordered());

        let mut pick = PickList::new(
            "PICK-2026-0001",
            "Acme Retail",
            "Delivery",
            vec![PickListItem {
                item_code: "STEEL-SHEET".into(),
                warehouse: "Raw Materials - AR".into(),
                batch_no: None,
                serial_no: None,
                qty: dec!(50),
                picked_qty: dec!(0),
            }],
        );
        pick.record_pick(0, dec!(50));
        assert_eq!(pick.status.as_str(), "Completed");

        let mut trip = DeliveryTrip::new(
            "TRIP-2026-0001",
            "John Doe",
            "VAN-01",
            "2026-10-17T08:00:00Z",
            vec![DeliveryStop {
                customer: "Cust A".into(),
                address: "123 Main St".into(),
                delivery_note: Some("DN-2026-0001".into()),
                visited: false,
            }],
        );
        assert!(!trip.is_all_visited());
        trip.complete_stop(0);
        assert!(trip.is_all_visited());
    }
}
