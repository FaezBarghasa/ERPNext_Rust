//! High-Throughput In-Memory Caching Subsystem (`frappe-net::cache`).
//!
//! Provides lock-free in-memory caches across all tenant workers for:
//! - Accounts Settings & Stock Settings
//! - Fiscal Years (cached on login to eliminate startup calls)
//! - Child Warehouses
//! - Pricing Rules (fetching only first matching rule)

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Cached Fiscal Year boundary definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedFiscalYear {
    pub year_name: CompactString,
    pub start_date: CompactString,
    pub end_date: CompactString,
    pub is_closed: bool,
}

/// Cached Pricing Rule definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedPricingRule {
    pub name: CompactString,
    pub item_code: Option<CompactString>,
    pub item_group: Option<CompactString>,
    pub customer: Option<CompactString>,
    pub min_qty: Decimal,
    pub discount_percentage: Decimal,
    pub priority: u32,
}

/// Lock-free tenant in-memory cache container.
#[derive(Default)]
pub struct TenantMemoryCache {
    accounts_settings: RwLock<HashMap<CompactString, serde_json::Value>>,
    stock_settings: RwLock<HashMap<CompactString, serde_json::Value>>,
    fiscal_years: RwLock<Vec<CachedFiscalYear>>,
    child_warehouses: RwLock<HashMap<CompactString, Vec<CompactString>>>,
    pricing_rules: RwLock<Vec<CachedPricingRule>>,
}

impl TenantMemoryCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets cached account settings.
    pub fn set_accounts_settings(&self, key: impl Into<CompactString>, val: serde_json::Value) {
        if let Ok(mut lock) = self.accounts_settings.write() {
            lock.insert(key.into(), val);
        }
    }

    /// Gets cached account setting.
    #[must_use]
    pub fn get_accounts_setting(&self, key: &str) -> Option<serde_json::Value> {
        if let Ok(lock) = self.accounts_settings.read() {
            lock.get(key).cloned()
        } else {
            None
        }
    }

    /// Sets cached stock settings.
    pub fn set_stock_settings(&self, key: impl Into<CompactString>, val: serde_json::Value) {
        if let Ok(mut lock) = self.stock_settings.write() {
            lock.insert(key.into(), val);
        }
    }

    /// Gets cached stock setting.
    #[must_use]
    pub fn get_stock_setting(&self, key: &str) -> Option<serde_json::Value> {
        if let Ok(lock) = self.stock_settings.read() {
            lock.get(key).cloned()
        } else {
            None
        }
    }

    /// Caches fiscal years on user login.
    pub fn set_fiscal_years(&self, years: Vec<CachedFiscalYear>) {
        if let Ok(mut lock) = self.fiscal_years.write() {
            *lock = years;
        }
    }

    /// Retrieves active fiscal year for a given date.
    #[must_use]
    pub fn get_fiscal_year_for_date(&self, date_str: &str) -> Option<CachedFiscalYear> {
        if let Ok(lock) = self.fiscal_years.read() {
            for fy in lock.iter() {
                if date_str >= fy.start_date.as_str() && date_str <= fy.end_date.as_str() {
                    return Some(fy.clone());
                }
            }
        }
        None
    }

    /// Caches child warehouse associations.
    pub fn set_child_warehouses(
        &self,
        parent: impl Into<CompactString>,
        children: Vec<CompactString>,
    ) {
        if let Ok(mut lock) = self.child_warehouses.write() {
            lock.insert(parent.into(), children);
        }
    }

    /// Gets child warehouses for a parent warehouse.
    #[must_use]
    pub fn get_child_warehouses(&self, parent: &str) -> Option<Vec<CompactString>> {
        if let Ok(lock) = self.child_warehouses.read() {
            lock.get(parent).cloned()
        } else {
            None
        }
    }

    /// Caches pricing rules sorted by priority descending.
    pub fn set_pricing_rules(&self, mut rules: Vec<CachedPricingRule>) {
        rules.sort_by_key(|a| std::cmp::Reverse(a.priority));
        if let Ok(mut lock) = self.pricing_rules.write() {
            *lock = rules;
        }
    }

    /// Finds the first matching pricing rule for the given item and customer.
    #[must_use]
    pub fn find_first_pricing_rule(
        &self,
        item_code: &str,
        customer: Option<&str>,
        qty: Decimal,
    ) -> Option<CachedPricingRule> {
        if let Ok(lock) = self.pricing_rules.read() {
            for rule in lock.iter() {
                if let Some(ref ic) = rule.item_code
                    && ic.as_str() != item_code
                {
                    continue;
                }
                if let (Some(rule_cust), Some(cust)) = (&rule.customer, customer)
                    && rule_cust.as_str() != cust
                {
                    continue;
                }
                if qty >= rule.min_qty {
                    return Some(rule.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tenant_memory_cache_fiscal_years_and_pricing_rules() {
        let cache = TenantMemoryCache::new();

        let fy = CachedFiscalYear {
            year_name: "2026-2027".into(),
            start_date: "2026-01-01".into(),
            end_date: "2026-12-31".into(),
            is_closed: false,
        };
        cache.set_fiscal_years(vec![fy]);

        let found = cache.get_fiscal_year_for_date("2026-06-15").unwrap();
        assert_eq!(found.year_name.as_str(), "2026-2027");

        let rule1 = CachedPricingRule {
            name: "RULE-01".into(),
            item_code: Some("SKU-100".into()),
            item_group: None,
            customer: Some("CUST-001".into()),
            min_qty: dec!(10),
            discount_percentage: dec!(15.0),
            priority: 10,
        };
        let rule2 = CachedPricingRule {
            name: "RULE-02".into(),
            item_code: Some("SKU-100".into()),
            item_group: None,
            customer: None,
            min_qty: dec!(1),
            discount_percentage: dec!(5.0),
            priority: 1,
        };

        cache.set_pricing_rules(vec![rule2, rule1]);

        let matched = cache
            .find_first_pricing_rule("SKU-100", Some("CUST-001"), dec!(12))
            .unwrap();
        assert_eq!(matched.name.as_str(), "RULE-01");
        assert_eq!(matched.discount_percentage, dec!(15.0));
    }
}
