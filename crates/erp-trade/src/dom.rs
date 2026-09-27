//! Distributed Order Management (DOM) & Fulfillment Routing Cost Optimizer.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FulfillmentNode {
    pub node_id: String,
    pub name: String,
    pub stock_on_hand: Decimal,
    pub picking_handling_cost: Decimal,
    pub tax_rate_percent: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShippingRateEstimate {
    pub node_id: String,
    pub carrier: String,
    pub freight_cost: Decimal,
    pub transit_days: u32,
}

pub struct DomRouter;

impl DomRouter {
    /// Selects optimal fulfillment node minimizing total landed cost (Freight + Handling + Tax).
    pub fn select_optimal_node<'a>(
        required_qty: Decimal,
        nodes: &'a [FulfillmentNode],
        shipping_estimates: &[ShippingRateEstimate],
        item_unit_value: Decimal,
    ) -> Option<(&'a FulfillmentNode, Decimal)> {
        let mut best: Option<(&'a FulfillmentNode, Decimal)> = None;

        for node in nodes {
            if node.stock_on_hand < required_qty {
                continue;
            }

            if let Some(shipping) = shipping_estimates
                .iter()
                .find(|s| s.node_id == node.node_id)
            {
                let handling = node.picking_handling_cost * required_qty;
                let tax =
                    (item_unit_value * required_qty) * (node.tax_rate_percent / Decimal::from(100));
                let total_cost = shipping.freight_cost + handling + tax;

                if best.is_none() || total_cost < best.unwrap().1 {
                    best = Some((node, total_cost));
                }
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_dom_routing_optimization() {
        let nodes = vec![
            FulfillmentNode {
                node_id: "WH-EAST".into(),
                name: "East Coast DC".into(),
                stock_on_hand: dec!(100),
                picking_handling_cost: dec!(2.00),
                tax_rate_percent: dec!(8.0),
            },
            FulfillmentNode {
                node_id: "WH-WEST".into(),
                name: "West Coast DC".into(),
                stock_on_hand: dec!(100),
                picking_handling_cost: dec!(3.00),
                tax_rate_percent: dec!(0.0), // Zero sales tax state
            },
        ];

        let shipping = vec![
            ShippingRateEstimate {
                node_id: "WH-EAST".into(),
                carrier: "UPS Ground".into(),
                freight_cost: dec!(25.00),
                transit_days: 2,
            },
            ShippingRateEstimate {
                node_id: "WH-WEST".into(),
                carrier: "FedEx Ground".into(),
                freight_cost: dec!(30.00),
                transit_days: 3,
            },
        ];

        let (best_node, cost) =
            DomRouter::select_optimal_node(dec!(10), &nodes, &shipping, dec!(50.00)).unwrap();
        // East: Freight 25 + Handling 20 + Tax (500*0.08=40) = 85
        // West: Freight 30 + Handling 30 + Tax 0 = 60
        assert_eq!(best_node.node_id, "WH-WEST");
        assert_eq!(cost, dec!(60.00));
    }
}
