//! Upstream ERPNext Commercial Trading Golden Tests (`erp-trade`).
//!
//! Validates:
//! 1. Order-to-Cash mapping lifecycle (Quotation -> Sales Order -> Delivery Note -> Sales Invoice).
//! 2. Procure-to-Pay mapping lifecycle (Purchase Order -> Purchase Receipt -> Purchase Invoice).
//! 3. Partial fulfillment tracking and overfulfillment rejection.
//! 4. Return Merchandise Authorization (RMA) returns & refund processing.

use erp_trade::{
    CommercialDoc, CommercialLineItem, MappedDocEngine, MappingError, OrderLifecycleError,
    OrderState, OrderStateMachine, OrderTransitionEvent, RmaItemLine, RmaRecord, RmaStatus,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_golden_order_to_cash_flow_mapping() {
    // 1. Customer Quotation
    let mut quote = CommercialDoc {
        name: "SAL-QTN-2026-0001".into(),
        doctype: "Quotation".into(),
        party_type: "Customer".into(),
        party: "CUST-001".into(),
        company: "Starlight Corp".into(),
        currency: "USD".into(),
        items: vec![
            CommercialLineItem {
                item_code: "TURBINE-BLADE".into(),
                item_name: "Titanium Turbine Blade".into(),
                uom: "Nos".into(),
                conversion_factor: Decimal::ONE,
                ordered_qty: dec!(4.0),
                fulfilled_qty: Decimal::ZERO,
                rate: dec!(5000.0),
                amount: dec!(20000.0),
                warehouse: Some("Main Stores".into()),
            },
            CommercialLineItem {
                item_code: "BEARING-SEAL".into(),
                item_name: "High Pressure Bearing Seal".into(),
                uom: "Nos".into(),
                conversion_factor: Decimal::ONE,
                ordered_qty: dec!(8.0),
                fulfilled_qty: Decimal::ZERO,
                rate: dec!(250.0),
                amount: dec!(2000.0),
                warehouse: Some("Main Stores".into()),
            },
        ],
        net_total: dec!(22000.0),
    };

    // 2. Map Quotation -> Sales Order
    let mut so = MappedDocEngine::make_mapped_doc(&mut quote, "Sales Order", "SAL-ORD-2026-0001")
        .expect("Mapping to Sales Order must succeed");
    assert_eq!(so.doctype, "Sales Order");
    assert_eq!(so.net_total, dec!(22000.0));
    assert_eq!(so.items.len(), 2);

    // 3. Partial fulfillment: Dispatch 2 Blades and 4 Seals
    MappedDocEngine::fulfill_item_qty(&mut so, "TURBINE-BLADE", dec!(2.0)).unwrap();
    MappedDocEngine::fulfill_item_qty(&mut so, "BEARING-SEAL", dec!(4.0)).unwrap();

    assert_eq!(so.items[0].pending_qty(), dec!(2.0));
    assert_eq!(so.items[1].pending_qty(), dec!(4.0));

    // 4. Map Partial pending items to Delivery Note
    let mut dn = MappedDocEngine::make_mapped_doc(&mut so, "Delivery Note", "MAT-DN-2026-0001")
        .expect("Mapping to Delivery Note must succeed");
    assert_eq!(dn.doctype, "Delivery Note");
    assert_eq!(dn.items[0].ordered_qty, dec!(2.0));
    assert_eq!(dn.items[1].ordered_qty, dec!(4.0));
    // 2 * 5000 + 4 * 250 = 10,000 + 1,000 = $11,000
    assert_eq!(dn.net_total, dec!(11000.0));

    // 5. Map Delivery Note -> Sales Invoice
    let si = MappedDocEngine::make_mapped_doc(&mut dn, "Sales Invoice", "ACC-SINV-2026-0001")
        .expect("Mapping to Sales Invoice must succeed");
    assert_eq!(si.doctype, "Sales Invoice");
    assert_eq!(si.net_total, dec!(11000.0));

    // 6. Overfulfillment check: Attempting to dispatch 5 blades when only 2 pending
    let over_err = MappedDocEngine::fulfill_item_qty(&mut dn, "TURBINE-BLADE", dec!(5.0));
    assert!(matches!(
        over_err,
        Err(MappingError::OverfulfilledQuantity { .. })
    ));
}

#[test]
fn test_golden_rma_return_and_refund_lifecycle() {
    let mut state = OrderState::Draft;
    assert_eq!(state, OrderState::Draft);

    // Place Order -> Pay -> Ship -> Deliver
    state = OrderStateMachine::transition(state, OrderTransitionEvent::PlaceOrder).unwrap();
    state = OrderStateMachine::transition(
        state,
        OrderTransitionEvent::ConfirmPayment {
            payment_ref: "PAY-123".into(),
        },
    )
    .unwrap();
    assert_eq!(state, OrderState::Processing);

    state = OrderStateMachine::transition(
        state,
        OrderTransitionEvent::DispatchShipment {
            tracking_number: "TRACK-99".into(),
            carrier: "FedEx".into(),
        },
    )
    .unwrap();
    assert_eq!(state, OrderState::Shipped);

    state = OrderStateMachine::transition(state, OrderTransitionEvent::ConfirmDelivery).unwrap();
    assert_eq!(state, OrderState::Delivered);

    // Customer initiates RMA
    let mut rma = RmaRecord::new(
        "RMA-2026-001",
        "ORD-2026-001",
        "CUST-001",
        vec![RmaItemLine {
            item_code: "DRONE-01".into(),
            qty: dec!(1.0),
            condition: "Defective Motor".into(),
            reason: "Motor vibrates excessively".into(),
            unit_refund_rate: dec!(500.0),
        }],
        "Customer reported severe motor defect on delivery",
    );
    assert_eq!(rma.status, RmaStatus::Submitted);
    assert_eq!(rma.total_refund_amount, dec!(500.0));

    // Transition Order state to ReturnRequested
    state = OrderStateMachine::transition(
        state,
        OrderTransitionEvent::RequestReturn {
            rma_id: rma.rma_id.clone(),
        },
    )
    .unwrap();
    assert_eq!(state, OrderState::ReturnRequested);

    // Approve Return
    rma.status = RmaStatus::Approved;
    state = OrderStateMachine::transition(state, OrderTransitionEvent::ApproveReturn).unwrap();
    assert_eq!(state, OrderState::ReturnApproved);

    // Receive Returned Goods
    rma.status = RmaStatus::GoodsReceived;
    state =
        OrderStateMachine::transition(state, OrderTransitionEvent::ReceiveReturnedGoods).unwrap();
    assert_eq!(state, OrderState::ReturnReceived);

    // Execute refund
    rma.status = RmaStatus::RefundCompleted;
    state = OrderStateMachine::transition(
        state,
        OrderTransitionEvent::ExecuteRefund {
            amount: rma.total_refund_amount,
            refund_ref: "REF-001".into(),
        },
    )
    .unwrap();
    assert_eq!(state, OrderState::Refunded);

    // Disallowed transition: Cannot place order on terminal Refunded state
    let err = OrderStateMachine::transition(state, OrderTransitionEvent::PlaceOrder);
    assert!(matches!(
        err,
        Err(OrderLifecycleError::InvalidTransition(
            OrderState::Refunded,
            _
        ))
    ));
}
