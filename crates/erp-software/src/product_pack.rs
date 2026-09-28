use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Visual inventory badge for digital storefront / e-commerce catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockBadge {
    /// Plentiful stock available.
    InStock { available_qty: u32 },
    /// Low stock alert badge.
    LowStock { remaining_qty: u32 },
    /// Sold out.
    OutOfStock,
    /// Out of stock but pre-orderable with expected delivery.
    PreOrderAvailable { expected_date: NaiveDate },
}

/// Component item bundled inside a Product Pack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackComponent {
    /// Item code.
    pub item_code: String,
    /// Required units per pack.
    pub qty_per_pack: u32,
    /// Individual standard unit price.
    pub standard_unit_price: Decimal,
    /// Current on-hand stock of this individual component.
    pub on_hand_stock: u32,
}

/// Pre-Order Engine configuration and limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreOrderConfig {
    /// Expected ship / release date.
    pub release_date: NaiveDate,
    /// Maximum pre-orders allowed before capping.
    pub max_pre_orders: u32,
    /// Number of pre-orders committed so far.
    pub current_pre_orders: u32,
    /// Upfront deposit required (percentage).
    pub deposit_percentage: Decimal,
}

impl PreOrderConfig {
    /// Checks whether additional pre-orders can be accepted.
    pub fn can_accept_pre_order(&self, requested_qty: u32) -> bool {
        self.current_pre_orders + requested_qty <= self.max_pre_orders
    }

    /// Computes required deposit amount for a pre-order.
    pub fn calculate_deposit(&self, total_order_amount: Decimal) -> Decimal {
        total_order_amount * (self.deposit_percentage / Decimal::from(100))
    }
}

/// Product Pack model with dynamic ribbon discount calculation and bundle availability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductPack {
    /// Pack item code.
    pub pack_code: String,
    /// Pack display title.
    pub pack_title: String,
    /// Bundled component lines.
    pub components: Vec<PackComponent>,
    /// Pack selling price.
    pub pack_price: Decimal,
    /// Pre-order settings (if product is in pre-order stage).
    pub pre_order: Option<PreOrderConfig>,
}

impl ProductPack {
    /// Calculates the standard unbundled total sum of individual items: `Σ (qty * unit_price)`.
    pub fn unbundled_total_value(&self) -> Decimal {
        self.components
            .iter()
            .map(|c| Decimal::from(c.qty_per_pack) * c.standard_unit_price)
            .sum()
    }

    /// Calculates savings percentage for promotional ribbon display: `((Unbundled - PackPrice) / Unbundled) * 100`.
    pub fn savings_percentage(&self) -> Decimal {
        let unbundled = self.unbundled_total_value();
        if unbundled <= Decimal::ZERO || self.pack_price >= unbundled {
            Decimal::ZERO
        } else {
            ((unbundled - self.pack_price) / unbundled) * Decimal::from(100)
        }
    }

    /// Computes how many complete packs can be assembled based on component stock levels (bottleneck calculation).
    pub fn max_available_packs(&self) -> u32 {
        if self.components.is_empty() {
            return 0;
        }

        self.components
            .iter()
            .map(|c| {
                if c.qty_per_pack == 0 {
                    0
                } else {
                    c.on_hand_stock / c.qty_per_pack
                }
            })
            .min()
            .unwrap_or(0)
    }

    /// Evaluates current stock badge for storefront rendering.
    pub fn resolve_stock_badge(&self, low_stock_threshold: u32) -> StockBadge {
        let available = self.max_available_packs();

        if available > low_stock_threshold {
            StockBadge::InStock {
                available_qty: available,
            }
        } else if available > 0 {
            StockBadge::LowStock {
                remaining_qty: available,
            }
        } else if let Some(pre) = &self.pre_order {
            if pre.can_accept_pre_order(1) {
                StockBadge::PreOrderAvailable {
                    expected_date: pre.release_date,
                }
            } else {
                StockBadge::OutOfStock
            }
        } else {
            StockBadge::OutOfStock
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_product_pack_savings_and_bottleneck_stock() {
        let pack = ProductPack {
            pack_code: "PACK-GAMING-TRIO".into(),
            pack_title: "Gaming Trio Bundle".into(),
            components: vec![
                PackComponent {
                    item_code: "MOUSE-RGB".into(),
                    qty_per_pack: 1,
                    standard_unit_price: dec!(50.00),
                    on_hand_stock: 10,
                },
                PackComponent {
                    item_code: "KEYBOARD-MECH".into(),
                    qty_per_pack: 1,
                    standard_unit_price: dec!(100.00),
                    on_hand_stock: 4, // Bottleneck: only 4 keyboards
                },
                PackComponent {
                    item_code: "HEADSET-71".into(),
                    qty_per_pack: 1,
                    standard_unit_price: dec!(50.00),
                    on_hand_stock: 20,
                },
            ],
            pack_price: dec!(160.00), // Standard = $200, Pack = $160 -> 20% discount
            pre_order: None,
        };

        assert_eq!(pack.unbundled_total_value(), dec!(200.00));
        assert_eq!(pack.savings_percentage(), dec!(20.00));
        assert_eq!(pack.max_available_packs(), 4);

        let badge = pack.resolve_stock_badge(5);
        assert_eq!(badge, StockBadge::LowStock { remaining_qty: 4 });
    }

    #[test]
    fn test_pre_order_engine() {
        let release = NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();
        let pre = PreOrderConfig {
            release_date: release,
            max_pre_orders: 100,
            current_pre_orders: 98,
            deposit_percentage: dec!(20.0), // 20% deposit
        };

        assert!(pre.can_accept_pre_order(2));
        assert!(!pre.can_accept_pre_order(3));
        assert_eq!(pre.calculate_deposit(dec!(500.00)), dec!(100.00));
    }
}
