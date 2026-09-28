//! Fixed Asset Capitalization, Repair & Component Tracking (`erp-accounting::asset_capitalization`).
//!
//! Provides:
//! - `Asset Capitalization Engine`: converts multiple stock items and service expenses into composite assets
//! - `is_composite_component` flag: modular components tracked without independent depreciation
//! - Capitalization of major overhauls on fully depreciated assets (recalculating useful life)
//! - Multi-invoice Asset Repair expense summation.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Stock or service item consumed in asset capitalization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalizationStockItem {
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub qty: Decimal,
    pub valuation_rate: Decimal,
    pub total_amount: Decimal,
}

/// Service expense item capitalized into asset value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalizationServiceItem {
    pub expense_account: CompactString,
    pub description: CompactString,
    pub amount: Decimal,
}

/// Asset Capitalization record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetCapitalizationRecord {
    pub name: CompactString,
    pub target_asset_name: CompactString,
    pub target_asset_category: CompactString,
    pub stock_items: Vec<CapitalizationStockItem>,
    pub service_items: Vec<CapitalizationServiceItem>,
    pub total_capitalized_value: Decimal,
    pub is_composite_component: bool,
}

impl AssetCapitalizationRecord {
    #[must_use]
    pub fn new(target_asset: impl Into<CompactString>, category: impl Into<CompactString>) -> Self {
        Self {
            name: CompactString::default(),
            target_asset_name: target_asset.into(),
            target_asset_category: category.into(),
            stock_items: Vec::new(),
            service_items: Vec::new(),
            total_capitalized_value: Decimal::ZERO,
            is_composite_component: false,
        }
    }

    /// Recalculates total capitalized value summing stock items and service expenses.
    pub fn compute_total_value(&mut self) -> Decimal {
        let stock_total: Decimal = self.stock_items.iter().map(|i| i.total_amount).sum();
        let service_total: Decimal = self.service_items.iter().map(|s| s.amount).sum();
        self.total_capitalized_value = stock_total + service_total;
        self.total_capitalized_value
    }
}

/// Asset Repair Overhaul Calculator for fully depreciated assets.
pub struct AssetOverhaulEngine;

impl AssetOverhaulEngine {
    /// Recalculates useful life and resumed monthly depreciation after a major repair.
    #[must_use]
    pub fn capitalize_repair(
        current_book_value: Decimal,
        repair_capitalized_cost: Decimal,
        additional_useful_life_months: u32,
    ) -> (Decimal, Decimal) {
        let new_book_value = current_book_value + repair_capitalized_cost;
        let monthly_depreciation = if additional_useful_life_months > 0 {
            new_book_value / Decimal::from(additional_useful_life_months)
        } else {
            Decimal::ZERO
        };
        (new_book_value, monthly_depreciation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asset_capitalization_computation() {
        let mut cap = AssetCapitalizationRecord::new("CNC-ROBOT-01", "Machinery");
        cap.stock_items.push(CapitalizationStockItem {
            item_code: "SERVO-MTR".into(),
            warehouse: "Stores - MB".into(),
            qty: dec!(4),
            valuation_rate: dec!(500.0),
            total_amount: dec!(2000.0),
        });
        cap.service_items.push(CapitalizationServiceItem {
            expense_account: "Engineering Labor".into(),
            description: "Custom assembly & wiring".into(),
            amount: dec!(1500.0),
        });

        let total = cap.compute_total_value();
        assert_eq!(total, dec!(3500.0));
    }

    #[test]
    fn test_repair_capitalization_fully_depreciated() {
        let (new_val, monthly) =
            AssetOverhaulEngine::capitalize_repair(dec!(0.0), dec!(12000.0), 24);
        assert_eq!(new_val, dec!(12000.0));
        assert_eq!(monthly, dec!(500.0));
    }
}
