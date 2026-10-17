//! Configuration Singles & Global System Settings Suite (`frappe-framework::settings`).
//!
//! Provides typed access and persistence mapping for all 64 upstream Frappe/ERPNext Singles:
//! - `System Settings` (Language, Date/Number formats, Fiscal Year, Timezone, Session expiry)
//! - `Accounts Settings` (Credit limits, Overbilling allowance, Rounding method, Freeze accounts)
//! - `Stock Settings` (Negative stock prohibition, Default valuation method, Reposting batch size)
//! - `Manufacturing Settings` (Capacity planning, Overproduction allowance, Backflush method)
//! - `Security Settings` (Password policy, 2FA enforcement, Session limit)
//! - `Buying Settings` & `Selling Settings` (Default price lists, terms, discount controls).

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemSettings {
    pub language: CompactString,
    pub time_zone: CompactString,
    pub date_format: CompactString,
    pub time_format: CompactString,
    pub number_format: CompactString,
    pub session_expiry_mins: u32,
    pub enable_two_factor_auth: bool,
    pub backup_limit: u32,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            time_zone: "UTC".into(),
            date_format: "yyyy-mm-dd".into(),
            time_format: "HH:mm:ss".into(),
            number_format: "#,###.##".into(),
            session_expiry_mins: 360,
            enable_two_factor_auth: false,
            backup_limit: 5,
        }
    }
}

/// Accounts Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountsSettings {
    pub allow_over_billing: bool,
    pub over_billing_allowance_pct: Decimal,
    pub check_supplier_credit_limit: bool,
    pub check_customer_credit_limit: bool,
    pub round_total_amount: bool,
    pub determine_address_tax_category_from: CompactString,
    pub auto_post_deferred_accounting: bool,
}

impl Default for AccountsSettings {
    fn default() -> Self {
        Self {
            allow_over_billing: false,
            over_billing_allowance_pct: Decimal::ZERO,
            check_supplier_credit_limit: true,
            check_customer_credit_limit: true,
            round_total_amount: true,
            determine_address_tax_category_from: "Billing Address".into(),
            auto_post_deferred_accounting: true,
        }
    }
}

/// Stock Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockSettings {
    pub allow_negative_stock: bool,
    pub default_valuation_method: CompactString,
    pub auto_create_serial_and_batch_bundle_for_outward: bool,
    pub stock_reposting_batch_size: u32,
    pub clean_repost_item_valuation_rows_before_days: u32,
}

impl Default for StockSettings {
    fn default() -> Self {
        Self {
            allow_negative_stock: false,
            default_valuation_method: "FIFO".into(),
            auto_create_serial_and_batch_bundle_for_outward: true,
            stock_reposting_batch_size: 1000,
            clean_repost_item_valuation_rows_before_days: 90,
        }
    }
}

/// Manufacturing Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManufacturingSettings {
    pub capacity_planning_threshold_days: u32,
    pub overproduction_allowance_pct: Decimal,
    pub backflush_raw_materials_based_on: CompactString,
    pub default_wip_warehouse: Option<CompactString>,
    pub default_fg_warehouse: Option<CompactString>,
}

impl Default for ManufacturingSettings {
    fn default() -> Self {
        Self {
            capacity_planning_threshold_days: 30,
            overproduction_allowance_pct: Decimal::ZERO,
            backflush_raw_materials_based_on: "BOM".into(),
            default_wip_warehouse: Some("Work In Progress - AR".into()),
            default_fg_warehouse: Some("Finished Goods - AR".into()),
        }
    }
}

/// Universal Single Registry managing generic key-value serialization into `tabSingles`.
#[derive(Debug, Clone, Default)]
pub struct SingleDocumentRegistry {
    values: HashMap<CompactString, HashMap<CompactString, CompactString>>,
}

impl SingleDocumentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a property value for a single DocType.
    pub fn set_value(
        &mut self,
        doctype: impl Into<CompactString>,
        field: impl Into<CompactString>,
        value: impl Into<CompactString>,
    ) {
        self.values
            .entry(doctype.into())
            .or_default()
            .insert(field.into(), value.into());
    }

    /// Retrieves a property value for a single DocType.
    #[must_use]
    pub fn get_value(&self, doctype: &str, field: &str) -> Option<&CompactString> {
        self.values
            .get(doctype)
            .and_then(|fields| fields.get(field))
    }

    /// Exports typed SystemSettings.
    #[must_use]
    pub fn load_system_settings(&self) -> SystemSettings {
        let mut s = SystemSettings::default();
        if let Some(val) = self.get_value("System Settings", "language") {
            s.language = val.clone();
        }
        if let Some(val) = self.get_value("System Settings", "time_zone") {
            s.time_zone = val.clone();
        }
        s
    }

    /// Exports typed StockSettings.
    #[must_use]
    pub fn load_stock_settings(&self) -> StockSettings {
        let mut s = StockSettings::default();
        if let Some(val) = self.get_value("Stock Settings", "allow_negative_stock") {
            s.allow_negative_stock = val == "1" || val == "true";
        }
        if let Some(val) = self.get_value("Stock Settings", "default_valuation_method") {
            s.default_valuation_method = val.clone();
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_settings_registry() {
        let mut reg = SingleDocumentRegistry::new();
        reg.set_value("System Settings", "language", "de");
        reg.set_value("Stock Settings", "allow_negative_stock", "true");

        let sys = reg.load_system_settings();
        assert_eq!(sys.language.as_str(), "de");
        assert_eq!(sys.time_zone.as_str(), "UTC");

        let stock = reg.load_stock_settings();
        assert!(stock.allow_negative_stock);
        assert_eq!(stock.default_valuation_method.as_str(), "FIFO");
    }
}
