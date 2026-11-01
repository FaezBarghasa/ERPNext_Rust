use erp_inventory::{InventoryError, add_fifo_layer, consume_fifo};
use proptest::prelude::*;
use rust_decimal::Decimal;

#[test]
fn test_fifo_layer_consumption_order() {
    let mut queue = Vec::new();
    // Layer 1: 10 units @ $5.00
    add_fifo_layer(&mut queue, Decimal::from(10), Decimal::from(5));
    // Layer 2: 20 units @ $8.00
    add_fifo_layer(&mut queue, Decimal::from(20), Decimal::from(8));

    // Consume 5 units: should come entirely from Layer 1 (5 * $5 = $25)
    let cost1 = consume_fifo(&mut queue, Decimal::from(5)).expect("Consumption failed");
    assert_eq!(cost1, Decimal::from(25));
    assert_eq!(queue.len(), 2);
    assert_eq!(queue[0].qty, Decimal::from(5));

    // Consume 10 units: 5 units from Layer 1 (5 * $5 = $25) + 5 units from Layer 2 (5 * $8 = $40) = $65
    let cost2 = consume_fifo(&mut queue, Decimal::from(10)).expect("Consumption failed");
    assert_eq!(cost2, Decimal::from(65));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].qty, Decimal::from(15));
}

#[test]
fn test_overconsume_rejected() {
    let mut queue = Vec::new();
    add_fifo_layer(&mut queue, Decimal::from(5), Decimal::from(10));

    let res = consume_fifo(&mut queue, Decimal::from(10));
    assert!(matches!(res, Err(InventoryError::InsufficientStock { .. })));
}

#[test]
fn test_zero_or_negative_qty_rejected() {
    let mut queue = Vec::new();
    add_fifo_layer(&mut queue, Decimal::from(5), Decimal::from(10));

    let res_zero = consume_fifo(&mut queue, Decimal::ZERO);
    assert!(matches!(
        res_zero,
        Err(InventoryError::InvalidQuantity(_))
    ));

    let res_neg = consume_fifo(&mut queue, Decimal::from(-5));
    assert!(matches!(res_neg, Err(InventoryError::InvalidQuantity(_))));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_total_fifo_cogs_invariance(
        q1 in 1..100i64, r1 in 1..50i64,
        q2 in 1..100i64, r2 in 1..50i64,
        q3 in 1..100i64, r3 in 1..50i64
    ) {
        let mut queue = Vec::new();
        add_fifo_layer(&mut queue, Decimal::from(q1), Decimal::from(r1));
        add_fifo_layer(&mut queue, Decimal::from(q2), Decimal::from(r2));
        add_fifo_layer(&mut queue, Decimal::from(q3), Decimal::from(r3));

        let total_expected_cogs = Decimal::from(q1 * r1 + q2 * r2 + q3 * r3);
        let total_qty = Decimal::from(q1 + q2 + q3);

        let consumed_cogs = consume_fifo(&mut queue, total_qty).unwrap();
        prop_assert_eq!(consumed_cogs, total_expected_cogs);
        prop_assert!(queue.is_empty(), "Queue must be empty after full consumption");
    }
}
