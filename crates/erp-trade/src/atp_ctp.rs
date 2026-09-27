//! Global Available-to-Promise (ATP) & Capable-to-Promise (CTP) Calculation Engine.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryPosition {
    pub item_code: String,
    pub stock_on_hand: Decimal,
    pub hard_allocations: Decimal,
    pub planned_factory_receipts: Decimal,
    pub safety_stock: Decimal,
    pub uncommitted_factory_capacity: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromiseAvailability {
    pub atp_quantity: Decimal, // Immediately deliverable
    pub ctp_quantity: Decimal, // Deliverable including on-demand manufacturing run
    pub can_fulfill_immediately: bool,
    pub can_fulfill_with_production: bool,
}

pub struct AtpCtpEngine;

impl AtpCtpEngine {
    #[must_use]
    pub fn evaluate_availability(
        pos: &InventoryPosition,
        requested_qty: Decimal,
    ) -> PromiseAvailability {
        // ATP = Stock On Hand - Hard Allocations + Planned Factory Receipts - Safety Stock
        let raw_atp = pos.stock_on_hand - pos.hard_allocations + pos.planned_factory_receipts
            - pos.safety_stock;
        let atp = raw_atp.max(Decimal::ZERO);

        // CTP = ATP + Uncommitted Work Center Capacity
        let ctp = atp + pos.uncommitted_factory_capacity;

        PromiseAvailability {
            atp_quantity: atp,
            ctp_quantity: ctp,
            can_fulfill_immediately: requested_qty <= atp,
            can_fulfill_with_production: requested_qty <= ctp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_atp_ctp_availability_computation() {
        let pos = InventoryPosition {
            item_code: "ROTOR-BLADE".into(),
            stock_on_hand: dec!(50),
            hard_allocations: dec!(20),
            planned_factory_receipts: dec!(10),
            safety_stock: dec!(5), // ATP = 50 - 20 + 10 - 5 = 35
            uncommitted_factory_capacity: dec!(100), // CTP = 35 + 100 = 135
        };

        let promise = AtpCtpEngine::evaluate_availability(&pos, dec!(50));
        assert_eq!(promise.atp_quantity, dec!(35));
        assert_eq!(promise.ctp_quantity, dec!(135));
        assert!(!promise.can_fulfill_immediately);
        assert!(promise.can_fulfill_with_production);
    }
}
