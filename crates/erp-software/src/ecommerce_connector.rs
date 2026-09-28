use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Target e-commerce channel platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ECommercePlatform {
    Shopify,
    WooCommerce,
    Magento,
}

/// Generic E-Commerce Product sync packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelProductPayload {
    /// Remote external channel product ID.
    pub external_id: String,
    /// Internal ERP item code.
    pub item_code: String,
    /// Product title.
    pub title: String,
    /// SKU identifier.
    pub sku: String,
    /// Selling price.
    pub price: Decimal,
    /// Inventory level to sync.
    pub stock_quantity: u32,
    /// Active status on channel.
    pub is_active: bool,
}

/// Generic E-Commerce Order line item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelOrderLineItem {
    pub sku: String,
    pub title: String,
    pub quantity: u32,
    pub unit_price: Decimal,
    pub total_amount: Decimal,
}

/// Inbound E-Commerce Order payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelOrderPayload {
    /// Remote order identifier.
    pub external_order_id: String,
    /// Channel platform.
    pub platform: ECommercePlatform,
    /// Customer email.
    pub customer_email: String,
    /// Customer full name.
    pub customer_name: String,
    /// Financial status (e.g. "paid", "pending").
    pub financial_status: String,
    /// Fulfillment status (e.g. "unfulfilled", "fulfilled").
    pub fulfillment_status: String,
    /// Currency code.
    pub currency: String,
    /// Total amount.
    pub total_amount: Decimal,
    /// Order line items.
    pub line_items: Vec<ChannelOrderLineItem>,
}

/// E-Commerce Channel Synchronization Engine.
pub struct ECommerceSyncEngine;

impl ECommerceSyncEngine {
    /// Prepares inventory update payload for pushing to external platform.
    pub fn build_inventory_update(
        item_code: &str,
        sku: &str,
        external_id: &str,
        available_qty: u32,
        unit_price: Decimal,
    ) -> ChannelProductPayload {
        ChannelProductPayload {
            external_id: external_id.to_string(),
            item_code: item_code.to_string(),
            title: String::new(),
            sku: sku.to_string(),
            price: unit_price,
            stock_quantity: available_qty,
            is_active: available_qty > 0,
        }
    }

    /// Validates and converts an inbound channel order into normalized totals.
    pub fn validate_order_totals(order: &ChannelOrderPayload) -> Result<Decimal, String> {
        let lines_total: Decimal = order.line_items.iter().map(|l| l.total_amount).sum();
        if lines_total != order.total_amount {
            return Err(format!(
                "Order total mismatch: lines sum {} != order total {}",
                lines_total, order.total_amount
            ));
        }
        Ok(lines_total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ecommerce_sync_and_validation() {
        let update = ECommerceSyncEngine::build_inventory_update(
            "LAPTOP-01",
            "SKU-LAP-01",
            "SHOPIFY-PROD-999",
            25,
            dec!(1200.00),
        );
        assert_eq!(update.stock_quantity, 25);
        assert!(update.is_active);

        let order = ChannelOrderPayload {
            external_order_id: "WOO-1001".into(),
            platform: ECommercePlatform::WooCommerce,
            customer_email: "cust@store.com".into(),
            customer_name: "John Store".into(),
            financial_status: "paid".into(),
            fulfillment_status: "unfulfilled".into(),
            currency: "USD".into(),
            total_amount: dec!(2400.00),
            line_items: vec![
                ChannelOrderLineItem {
                    sku: "SKU-LAP-01".into(),
                    title: "Laptop".into(),
                    quantity: 2,
                    unit_price: dec!(1200.00),
                    total_amount: dec!(2400.00),
                },
            ],
        };

        assert_eq!(
            ECommerceSyncEngine::validate_order_totals(&order).unwrap(),
            dec!(2400.00)
        );
    }
}
