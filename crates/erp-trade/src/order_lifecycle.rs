//! Order Lifecycle & Return Merchandise Authorization (RMA) Engine (`erp-trade::order_lifecycle`).
//!
//! Provides deterministic state machines for order processing, shipping, cancellation,
//! and complete RMA return & refund cycles matching Odoo and WooCommerce standards.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Order lifecycle and transition errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrderLifecycleError {
    #[error("Invalid state transition from '{0:?}' with event '{1}'")]
    InvalidTransition(OrderState, String),
    #[error("RMA '{0}' cannot be processed in status '{1:?}'")]
    InvalidRmaStatus(String, RmaStatus),
    #[error("Refund amount {0} exceeds eligible refundable balance of {1}")]
    RefundExceedsBalance(Decimal, Decimal),
}

/// Order state in the fulfillment pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    Draft,
    PendingPayment,
    Processing,
    PartiallyShipped,
    Shipped,
    Delivered,
    Cancelled,
    ReturnRequested,
    ReturnApproved,
    ReturnReceived,
    PartiallyRefunded,
    Refunded,
    Disputed,
}

/// Event triggering an order state transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderTransitionEvent {
    PlaceOrder,
    ConfirmPayment {
        payment_ref: String,
    },
    StartProcessing,
    DispatchShipment {
        tracking_number: String,
        carrier: String,
    },
    ConfirmDelivery,
    CancelOrder {
        reason: String,
    },
    RequestReturn {
        rma_id: String,
    },
    ApproveReturn,
    ReceiveReturnedGoods,
    ExecuteRefund {
        amount: Decimal,
        refund_ref: String,
    },
    OpenDispute {
        reason: String,
    },
}

/// Return Merchandise Authorization (RMA) item line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RmaItemLine {
    pub item_code: String,
    pub qty: Decimal,
    pub condition: String, // e.g. "Unopened", "Defective", "Damaged in Transit"
    pub reason: String,
    pub unit_refund_rate: Decimal,
}

/// RMA Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RmaStatus {
    Submitted,
    Approved,
    GoodsReceived,
    Inspected,
    RefundCompleted,
    Rejected,
}

/// Return Merchandise Authorization (RMA) Record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RmaRecord {
    pub rma_id: String,
    pub order_id: String,
    pub customer_id: String,
    pub items: Vec<RmaItemLine>,
    pub general_reason: String,
    pub status: RmaStatus,
    pub total_refund_amount: Decimal,
    pub return_tracking_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RmaRecord {
    #[must_use]
    pub fn new(
        rma_id: impl Into<String>,
        order_id: impl Into<String>,
        customer_id: impl Into<String>,
        items: Vec<RmaItemLine>,
        reason: impl Into<String>,
    ) -> Self {
        let total_refund: Decimal = items.iter().map(|i| i.qty * i.unit_refund_rate).sum();

        let now = Utc::now();
        Self {
            rma_id: rma_id.into(),
            order_id: order_id.into(),
            customer_id: customer_id.into(),
            items,
            general_reason: reason.into(),
            status: RmaStatus::Submitted,
            total_refund_amount: total_refund,
            return_tracking_number: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// State Machine handling order lifecycle transitions.
pub struct OrderStateMachine;

impl OrderStateMachine {
    /// Executes a deterministic state transition.
    pub fn transition(
        current: OrderState,
        event: OrderTransitionEvent,
    ) -> Result<OrderState, OrderLifecycleError> {
        match (current, event) {
            // Draft -> Pending Payment
            (OrderState::Draft, OrderTransitionEvent::PlaceOrder) => Ok(OrderState::PendingPayment),

            // PendingPayment -> Processing
            (OrderState::PendingPayment, OrderTransitionEvent::ConfirmPayment { .. }) => {
                Ok(OrderState::Processing)
            }

            // PendingPayment -> Cancelled
            (OrderState::PendingPayment, OrderTransitionEvent::CancelOrder { .. }) => {
                Ok(OrderState::Cancelled)
            }

            // Processing -> Shipped
            (OrderState::Processing, OrderTransitionEvent::DispatchShipment { .. }) => {
                Ok(OrderState::Shipped)
            }

            // Processing -> Cancelled (if not yet fulfilled)
            (OrderState::Processing, OrderTransitionEvent::CancelOrder { .. }) => {
                Ok(OrderState::Cancelled)
            }

            // Shipped -> Delivered
            (OrderState::Shipped, OrderTransitionEvent::ConfirmDelivery) => {
                Ok(OrderState::Delivered)
            }

            // Delivered -> ReturnRequested
            (OrderState::Delivered, OrderTransitionEvent::RequestReturn { .. }) => {
                Ok(OrderState::ReturnRequested)
            }

            // ReturnRequested -> ReturnApproved
            (OrderState::ReturnRequested, OrderTransitionEvent::ApproveReturn) => {
                Ok(OrderState::ReturnApproved)
            }

            // ReturnApproved -> ReturnReceived
            (OrderState::ReturnApproved, OrderTransitionEvent::ReceiveReturnedGoods) => {
                Ok(OrderState::ReturnReceived)
            }

            // ReturnReceived -> Refunded
            (OrderState::ReturnReceived, OrderTransitionEvent::ExecuteRefund { .. }) => {
                Ok(OrderState::Refunded)
            }

            // Cancelled -> Refunded (if payment was previously captured)
            (OrderState::Cancelled, OrderTransitionEvent::ExecuteRefund { .. }) => {
                Ok(OrderState::Refunded)
            }

            // Any active order -> Disputed
            (
                OrderState::Processing | OrderState::Shipped | OrderState::Delivered,
                OrderTransitionEvent::OpenDispute { .. },
            ) => Ok(OrderState::Disputed),

            (state, ev) => Err(OrderLifecycleError::InvalidTransition(
                state,
                format!("{ev:?}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_standard_order_lifecycle_to_delivery() {
        let mut state = OrderState::Draft;

        state = OrderStateMachine::transition(state, OrderTransitionEvent::PlaceOrder).unwrap();
        assert_eq!(state, OrderState::PendingPayment);

        state = OrderStateMachine::transition(
            state,
            OrderTransitionEvent::ConfirmPayment {
                payment_ref: "PAY_12345".into(),
            },
        )
        .unwrap();
        assert_eq!(state, OrderState::Processing);

        state = OrderStateMachine::transition(
            state,
            OrderTransitionEvent::DispatchShipment {
                tracking_number: "TRACK-987".into(),
                carrier: "DHL Express".into(),
            },
        )
        .unwrap();
        assert_eq!(state, OrderState::Shipped);

        state =
            OrderStateMachine::transition(state, OrderTransitionEvent::ConfirmDelivery).unwrap();
        assert_eq!(state, OrderState::Delivered);
    }

    #[test]
    fn test_rma_return_and_refund_lifecycle() {
        let mut state = OrderState::Delivered;

        state = OrderStateMachine::transition(
            state,
            OrderTransitionEvent::RequestReturn {
                rma_id: "RMA-001".into(),
            },
        )
        .unwrap();
        assert_eq!(state, OrderState::ReturnRequested);

        state = OrderStateMachine::transition(state, OrderTransitionEvent::ApproveReturn).unwrap();
        assert_eq!(state, OrderState::ReturnApproved);

        state = OrderStateMachine::transition(state, OrderTransitionEvent::ReceiveReturnedGoods)
            .unwrap();
        assert_eq!(state, OrderState::ReturnReceived);

        state = OrderStateMachine::transition(
            state,
            OrderTransitionEvent::ExecuteRefund {
                amount: dec!(120.00),
                refund_ref: "REF_BANK_001".into(),
            },
        )
        .unwrap();
        assert_eq!(state, OrderState::Refunded);

        // Disallowed transition from Refunded
        let err = OrderStateMachine::transition(state, OrderTransitionEvent::PlaceOrder);
        assert!(matches!(
            err,
            Err(OrderLifecycleError::InvalidTransition(
                OrderState::Refunded,
                _
            ))
        ));
    }
}
