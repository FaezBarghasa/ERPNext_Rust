use crate::fifo::InventoryError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source document type requesting stock reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReservationType {
    /// Sales Order line item.
    SalesOrder,
    /// Production Plan material requirement.
    ProductionPlan,
    /// Work Order component allocation.
    WorkOrder,
    /// Subcontracting Order provided item.
    SubcontractingOrder,
    /// Point of Sale active draft cart hold.
    PosHold,
}

/// Lifecycle status of a stock reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationStatus {
    /// Draft reservation pending validation.
    Draft,
    /// Actively holding physical or future stock.
    Reserved,
    /// Partially fulfilled against delivery or material transfer.
    PartiallyDelivered,
    /// Fully consumed and fulfilled.
    Delivered,
    /// Cancelled or manually released.
    Cancelled,
}

/// Stock Reservation Entry (SRE) representing allocated stock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockReservationEntry {
    /// Unique reservation identifier.
    pub name: String,
    /// Originating document type.
    pub voucher_type: ReservationType,
    /// Originating document identifier.
    pub voucher_no: String,
    /// Originating line item / detail row identifier.
    pub voucher_detail_no: String,
    /// Target Item code.
    pub item_code: String,
    /// Target Warehouse.
    pub warehouse: String,
    /// Specific allocated Batch (if batch-tracked).
    pub batch_no: Option<String>,
    /// Quantity reserved.
    pub reserved_qty: Decimal,
    /// Quantity consumed/delivered so far.
    pub delivered_qty: Decimal,
    /// Current reservation status.
    pub status: ReservationStatus,
    /// Flag indicating whether this is an ephemeral POS cart hold.
    pub is_pos_hold: bool,
}

impl StockReservationEntry {
    /// Remaining unfulfilled reserved quantity.
    pub fn remaining_reserved_qty(&self) -> Decimal {
        if self.status == ReservationStatus::Cancelled
            || self.status == ReservationStatus::Delivered
        {
            Decimal::ZERO
        } else {
            (self.reserved_qty - self.delivered_qty).max(Decimal::ZERO)
        }
    }
}

/// Universal Stock Reservation Engine managing allocations across sales, manufacturing, and POS.
#[derive(Debug, Default, Clone)]
pub struct StockReservationEngine {
    /// Active reservations indexed by reservation entry ID.
    pub reservations: HashMap<String, StockReservationEntry>,
}

impl StockReservationEngine {
    /// Creates a new empty stock reservation engine.
    pub fn new() -> Self {
        Self {
            reservations: HashMap::new(),
        }
    }

    /// Calculates total currently reserved quantity for an item + warehouse (+ optional batch).
    pub fn get_total_reserved_qty(
        &self,
        item_code: &str,
        warehouse: &str,
        batch_no: Option<&str>,
        include_pos_holds: bool,
    ) -> Decimal {
        self.reservations
            .values()
            .filter(|r| {
                if !include_pos_holds && r.is_pos_hold {
                    return false;
                }
                if r.status != ReservationStatus::Reserved
                    && r.status != ReservationStatus::PartiallyDelivered
                {
                    return false;
                }
                if r.item_code != item_code || r.warehouse != warehouse {
                    return false;
                }
                if let Some(target_batch) = batch_no {
                    r.batch_no.as_deref() == Some(target_batch)
                } else {
                    true
                }
            })
            .map(|r| r.remaining_reserved_qty())
            .sum()
    }

    /// Validates and reserves stock against available on-hand inventory.
    pub fn reserve_stock(
        &mut self,
        mut entry: StockReservationEntry,
        actual_on_hand_qty: Decimal,
    ) -> Result<String, InventoryError> {
        if entry.reserved_qty <= Decimal::ZERO {
            return Err(InventoryError::InvalidQuantity(entry.reserved_qty));
        }

        // Calculate already reserved stock excluding this entry if updating
        let already_reserved = self.get_total_reserved_qty(
            &entry.item_code,
            &entry.warehouse,
            entry.batch_no.as_deref(),
            true,
        );

        let available = actual_on_hand_qty - already_reserved;

        if available < entry.reserved_qty {
            return Err(InventoryError::InsufficientStock {
                requested: entry.reserved_qty,
                available: available.max(Decimal::ZERO),
            });
        }

        entry.status = ReservationStatus::Reserved;
        let id = entry.name.clone();
        self.reservations.insert(id.clone(), entry);
        Ok(id)
    }

    /// Releases or reduces reserved quantity on an active reservation.
    pub fn release_reservation(
        &mut self,
        reservation_id: &str,
        release_qty: Option<Decimal>,
    ) -> Result<(), InventoryError> {
        let entry = self
            .reservations
            .get_mut(reservation_id)
            .ok_or_else(|| InventoryError::ItemNotFound(reservation_id.to_string()))?;

        if entry.status == ReservationStatus::Cancelled
            || entry.status == ReservationStatus::Delivered
        {
            return Ok(());
        }

        if let Some(qty) = release_qty {
            if qty <= Decimal::ZERO {
                return Err(InventoryError::InvalidQuantity(qty));
            }
            let remaining = entry.remaining_reserved_qty();
            if qty >= remaining {
                entry.status = ReservationStatus::Cancelled;
            } else {
                entry.reserved_qty -= qty;
            }
        } else {
            entry.status = ReservationStatus::Cancelled;
        }

        Ok(())
    }

    /// Consumes reserved stock upon actual shipment/delivery voucher posting.
    pub fn consume_reservation(
        &mut self,
        reservation_id: &str,
        delivered_qty: Decimal,
    ) -> Result<(), InventoryError> {
        if delivered_qty <= Decimal::ZERO {
            return Err(InventoryError::InvalidQuantity(delivered_qty));
        }

        let entry = self
            .reservations
            .get_mut(reservation_id)
            .ok_or_else(|| InventoryError::ItemNotFound(reservation_id.to_string()))?;

        entry.delivered_qty += delivered_qty;
        if entry.delivered_qty >= entry.reserved_qty {
            entry.status = ReservationStatus::Delivered;
        } else {
            entry.status = ReservationStatus::PartiallyDelivered;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_stock_reservation_lifecycle() {
        let mut engine = StockReservationEngine::new();

        let sre = StockReservationEntry {
            name: "SRE-0001".into(),
            voucher_type: ReservationType::SalesOrder,
            voucher_no: "SO-2026-001".into(),
            voucher_detail_no: "SO-ITEM-01".into(),
            item_code: "LAPTOP-X".into(),
            warehouse: "Finished Goods - ACME".into(),
            batch_no: Some("BATCH-100".into()),
            reserved_qty: dec!(10.0),
            delivered_qty: dec!(0.0),
            status: ReservationStatus::Draft,
            is_pos_hold: false,
        };

        // Available on hand is 15 -> Reserving 10 succeeds
        let id = engine
            .reserve_stock(sre, dec!(15.0))
            .expect("Reserve failed");
        assert_eq!(id, "SRE-0001");

        // Now reserved is 10, available is 5
        let reserved = engine.get_total_reserved_qty(
            "LAPTOP-X",
            "Finished Goods - ACME",
            Some("BATCH-100"),
            true,
        );
        assert_eq!(reserved, dec!(10.0));

        // Attempt reserving 6 more -> Fails due to insufficient available (only 5 left)
        let sre_over = StockReservationEntry {
            name: "SRE-0002".into(),
            voucher_type: ReservationType::WorkOrder,
            voucher_no: "WO-2026-001".into(),
            voucher_detail_no: "WO-ITEM-01".into(),
            item_code: "LAPTOP-X".into(),
            warehouse: "Finished Goods - ACME".into(),
            batch_no: Some("BATCH-100".into()),
            reserved_qty: dec!(6.0),
            delivered_qty: dec!(0.0),
            status: ReservationStatus::Draft,
            is_pos_hold: false,
        };
        assert!(engine.reserve_stock(sre_over, dec!(15.0)).is_err());

        // Partially deliver 4 units
        engine
            .consume_reservation("SRE-0001", dec!(4.0))
            .expect("Consume failed");
        let entry = engine.reservations.get("SRE-0001").unwrap();
        assert_eq!(entry.status, ReservationStatus::PartiallyDelivered);
        assert_eq!(entry.remaining_reserved_qty(), dec!(6.0));

        // Deliver remaining 6 units
        engine
            .consume_reservation("SRE-0001", dec!(6.0))
            .expect("Consume failed");
        let entry = engine.reservations.get("SRE-0001").unwrap();
        assert_eq!(entry.status, ReservationStatus::Delivered);
        assert_eq!(entry.remaining_reserved_qty(), dec!(0.0));
    }
}
