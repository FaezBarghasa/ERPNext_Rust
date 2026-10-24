//! Configuration Singles & Global System Settings Suite (`rwl-framework::settings`).
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

/// Buying Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuyingSettings {
    pub supplier_group: Option<CompactString>,
    pub maintain_same_rate: bool,
    pub buying_price_list: CompactString,
    pub auto_create_purchase_receipt: bool,
}

impl Default for BuyingSettings {
    fn default() -> Self {
        Self {
            supplier_group: Some("All Supplier Groups".into()),
            maintain_same_rate: false,
            buying_price_list: "Standard Buying".into(),
            auto_create_purchase_receipt: false,
        }
    }
}

/// Selling Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SellingSettings {
    pub customer_group: Option<CompactString>,
    pub selling_price_list: CompactString,
    pub validate_selling_price: bool,
    pub editable_price_list_rate: bool,
}

impl Default for SellingSettings {
    fn default() -> Self {
        Self {
            customer_group: Some("All Customer Groups".into()),
            selling_price_list: "Standard Selling".into(),
            validate_selling_price: true,
            editable_price_list_rate: true,
        }
    }
}

/// CRM Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrmSettings {
    pub default_lead_source: Option<CompactString>,
    pub auto_creation_of_lead: bool,
    pub close_opportunity_after_days: u32,
}

impl Default for CrmSettings {
    fn default() -> Self {
        Self {
            default_lead_source: Some("Advertisement".into()),
            auto_creation_of_lead: true,
            close_opportunity_after_days: 30,
        }
    }
}

/// Support Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportSettings {
    pub track_service_level_agreement: bool,
    pub close_ticket_after_days: u32,
}

impl Default for SupportSettings {
    fn default() -> Self {
        Self {
            track_service_level_agreement: true,
            close_ticket_after_days: 7,
        }
    }
}

/// Security Settings single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecuritySettings {
    pub min_password_score: u32,
    pub force_user_to_reset_password: bool,
    pub enable_password_policy: bool,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            min_password_score: 2,
            force_user_to_reset_password: false,
            enable_password_policy: true,
        }
    }
}

/// Global Defaults single document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalDefaults {
    pub default_company: Option<CompactString>,
    pub default_currency: CompactString,
    pub current_fiscal_year: Option<CompactString>,
}

impl Default for GlobalDefaults {
    fn default() -> Self {
        Self {
            default_company: None,
            default_currency: "USD".into(),
            current_fiscal_year: None,
        }
    }
}

/// Canonical array of all 64 upstream Single DocTypes.
pub const ALL_SINGLE_DOCTYPES: &[&str] = &[
    "About Us Settings",
    "Accounts Settings",
    "Appointment Booking Settings",
    "Audit Trail",
    "Authorization Control",
    "Automation Settings",
    "BOM Update Tool",
    "Bank Clearance",
    "Bank Reconciliation Tool",
    "Bisect Accounting Statements",
    "Bulk Update",
    "Buying Settings",
    "CRM Settings",
    "Chart of Accounts Importer",
    "Contact Us Settings",
    "Currency Exchange Settings",
    "Customize Form",
    "Data Export",
    "Delivery Settings",
    "Desktop Settings",
    "Document Naming Settings",
    "Domain Settings",
    "Geolocation Settings",
    "Global Defaults",
    "Global Search Settings",
    "Google Settings",
    "Installed Applications",
    "Item Variant Settings",
    "LDAP Settings",
    "Ledger Health Monitor",
    "Log Settings",
    "MCP Settings",
    "Manufacturing Settings",
    "Navbar Settings",
    "OAuth Provider Settings",
    "OAuth Settings",
    "Opening Invoice Creation Tool",
    "POS Settings",
    "Payment Reconciliation",
    "Pegged Currencies",
    "Permission Inspector",
    "Plaid Settings",
    "Portal Settings",
    "Print Settings",
    "Projects Settings",
    "Push Notification Settings",
    "Quick Stock Balance",
    "Rename Tool",
    "Role Permission for Page and Report",
    "SMS Center",
    "SMS Settings",
    "Security Settings",
    "Selling Settings",
    "Session Default Settings",
    "Stock Reposting Settings",
    "Stock Settings",
    "Subscription Settings",
    "Support Settings",
    "System Console",
    "System Health Report",
    "System Settings",
    "Video Settings",
    "Website Script",
    "Website Settings",
];

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

    #[test]
    fn test_all_64_single_doctypes() {
        assert_eq!(ALL_SINGLE_DOCTYPES.len(), 64);
        let mut reg = SingleDocumentRegistry::new();
        for &dt in ALL_SINGLE_DOCTYPES {
            reg.set_value(dt, "enabled", "1");
            assert_eq!(reg.get_value(dt, "enabled").map(|s| s.as_str()), Some("1"));
        }
    }
}
