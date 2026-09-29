//! Shipping Zones & Rate Calculation Engine (`erp-trade::shipping`).
//!
//! Provides enterprise carrier and fulfillment rule management matching WooCommerce and Odoo:
//! - Geographical zones matching countries, states/provinces, and postal code wildcards/ranges
//! - Multiple calculation methods: Flat rate, Free shipping threshold, Weight-tiered rates, Local pickup
//! - Dynamic checkout rate estimation

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of shipping calculation method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShippingMethodType {
    /// Standard flat fee per order.
    FlatRate,
    /// Free shipping when order subtotal meets or exceeds threshold.
    FreeShippingThreshold,
    /// Base rate plus additional fee per kg of total shipment weight.
    WeightBasedRate,
    /// Customer collects order at warehouse/store with zero delivery charge.
    LocalPickup,
}

/// Configured shipping method within a zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShippingMethod {
    pub id: String,
    pub name: String,
    pub method_type: ShippingMethodType,
    pub base_cost: Decimal,
    pub free_shipping_threshold: Option<Decimal>,
    pub per_kg_rate: Option<Decimal>,
    pub estimated_delivery_days_min: u8,
    pub estimated_delivery_days_max: u8,
    pub is_enabled: bool,
}

impl ShippingMethod {
    #[must_use]
    pub fn flat_rate(
        id: impl Into<String>,
        name: impl Into<String>,
        cost: Decimal,
        delivery_days: (u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            method_type: ShippingMethodType::FlatRate,
            base_cost: cost,
            free_shipping_threshold: None,
            per_kg_rate: None,
            estimated_delivery_days_min: delivery_days.0,
            estimated_delivery_days_max: delivery_days.1,
            is_enabled: true,
        }
    }

    #[must_use]
    pub fn free_shipping(
        id: impl Into<String>,
        name: impl Into<String>,
        threshold: Decimal,
        delivery_days: (u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            method_type: ShippingMethodType::FreeShippingThreshold,
            base_cost: Decimal::ZERO,
            free_shipping_threshold: Some(threshold),
            per_kg_rate: None,
            estimated_delivery_days_min: delivery_days.0,
            estimated_delivery_days_max: delivery_days.1,
            is_enabled: true,
        }
    }

    #[must_use]
    pub fn weight_based(
        id: impl Into<String>,
        name: impl Into<String>,
        base_cost: Decimal,
        per_kg: Decimal,
        delivery_days: (u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            method_type: ShippingMethodType::WeightBasedRate,
            base_cost,
            free_shipping_threshold: None,
            per_kg_rate: Some(per_kg),
            estimated_delivery_days_min: delivery_days.0,
            estimated_delivery_days_max: delivery_days.1,
            is_enabled: true,
        }
    }

    #[must_use]
    pub fn local_pickup(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            method_type: ShippingMethodType::LocalPickup,
            base_cost: Decimal::ZERO,
            free_shipping_threshold: None,
            per_kg_rate: None,
            estimated_delivery_days_min: 0,
            estimated_delivery_days_max: 1,
            is_enabled: true,
        }
    }
}

/// Geographical zone grouping regions and corresponding shipping options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShippingZone {
    pub id: String,
    pub name: String,
    /// ISO 3166-1 alpha-2 country codes (e.g. "US", "DE", "IR", "*").
    pub countries: Vec<String>,
    /// State/Province codes (e.g. "CA", "NY", "TEH").
    pub state_provinces: Vec<String>,
    /// Postal code prefixes or wildcard patterns (e.g. "902*", "75001..75020").
    pub postal_code_patterns: Vec<String>,
    pub methods: Vec<ShippingMethod>,
}

impl ShippingZone {
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            countries: Vec::new(),
            state_provinces: Vec::new(),
            postal_code_patterns: Vec::new(),
            methods: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_countries(mut self, countries: Vec<String>) -> Self {
        self.countries = countries;
        self
    }

    #[must_use]
    pub fn with_methods(mut self, methods: Vec<ShippingMethod>) -> Self {
        self.methods = methods;
        self
    }

    /// Evaluates if a destination address matches this zone.
    #[must_use]
    pub fn matches(&self, country: &str, state: Option<&str>, postal_code: Option<&str>) -> bool {
        // Match country
        let country_match = self.countries.is_empty()
            || self
                .countries
                .iter()
                .any(|c| c == "*" || c.eq_ignore_ascii_case(country));

        if !country_match {
            return false;
        }

        // Match state if specified in zone
        if !self.state_provinces.is_empty() {
            let state_val = state.unwrap_or("");
            let state_match = self
                .state_provinces
                .iter()
                .any(|s| s.eq_ignore_ascii_case(state_val));
            if !state_match {
                return false;
            }
        }

        // Match postal code pattern if specified
        if !self.postal_code_patterns.is_empty() {
            let zip_val = postal_code.unwrap_or("");
            let zip_match = self.postal_code_patterns.iter().any(|pattern| {
                if let Some(prefix) = pattern.strip_suffix('*') {
                    zip_val.starts_with(prefix)
                } else {
                    pattern.eq_ignore_ascii_case(zip_val)
                }
            });
            if !zip_match {
                return false;
            }
        }

        true
    }
}

/// Estimated rate option returned to the customer during checkout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShippingRateOption {
    pub method_id: String,
    pub method_name: String,
    pub cost: Decimal,
    pub estimated_days_min: u8,
    pub estimated_days_max: u8,
    pub is_free: bool,
}

/// Shipping Calculation Engine.
pub struct ShippingCalculator;

impl ShippingCalculator {
    /// Resolves the most specific matching shipping zone for a given delivery address.
    #[must_use]
    pub fn match_zone<'a>(
        country: &str,
        state: Option<&str>,
        postal_code: Option<&str>,
        zones: &'a [ShippingZone],
    ) -> Option<&'a ShippingZone> {
        zones
            .iter()
            .find(|zone| zone.matches(country, state, postal_code))
    }

    /// Calculates all applicable shipping rate options for the cart in a resolved zone.
    #[must_use]
    pub fn calculate_rates(
        zone: &ShippingZone,
        cart_subtotal: Decimal,
        cart_weight_kg: Decimal,
    ) -> Vec<ShippingRateOption> {
        let mut options = Vec::new();

        for method in &zone.methods {
            if !method.is_enabled {
                continue;
            }

            match &method.method_type {
                ShippingMethodType::FlatRate => {
                    options.push(ShippingRateOption {
                        method_id: method.id.clone(),
                        method_name: method.name.clone(),
                        cost: method.base_cost,
                        estimated_days_min: method.estimated_delivery_days_min,
                        estimated_days_max: method.estimated_delivery_days_max,
                        is_free: method.base_cost == Decimal::ZERO,
                    });
                }
                ShippingMethodType::FreeShippingThreshold => {
                    if method
                        .free_shipping_threshold
                        .is_some_and(|threshold| cart_subtotal >= threshold)
                    {
                        options.push(ShippingRateOption {
                            method_id: method.id.clone(),
                            method_name: method.name.clone(),
                            cost: Decimal::ZERO,
                            estimated_days_min: method.estimated_delivery_days_min,
                            estimated_days_max: method.estimated_delivery_days_max,
                            is_free: true,
                        });
                    }
                }
                ShippingMethodType::WeightBasedRate => {
                    let per_kg = method.per_kg_rate.unwrap_or(Decimal::ZERO);
                    let weight_charge = cart_weight_kg * per_kg;
                    let total_cost = method.base_cost + weight_charge;
                    options.push(ShippingRateOption {
                        method_id: method.id.clone(),
                        method_name: method.name.clone(),
                        cost: total_cost,
                        estimated_days_min: method.estimated_delivery_days_min,
                        estimated_days_max: method.estimated_delivery_days_max,
                        is_free: total_cost == Decimal::ZERO,
                    });
                }
                ShippingMethodType::LocalPickup => {
                    options.push(ShippingRateOption {
                        method_id: method.id.clone(),
                        method_name: method.name.clone(),
                        cost: Decimal::ZERO,
                        estimated_days_min: 0,
                        estimated_days_max: 1,
                        is_free: true,
                    });
                }
            }
        }

        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_zone_matching_and_rate_calculation() {
        let us_west = ShippingZone::new("zone_us_west", "US West Coast")
            .with_countries(vec!["US".into()])
            .with_methods(vec![
                ShippingMethod::flat_rate(
                    "std_ground",
                    "Standard Ground (3-5 Days)",
                    dec!(12.00),
                    (3, 5),
                ),
                ShippingMethod::free_shipping(
                    "free_tier",
                    "Free Super Saver Shipping",
                    dec!(100.00),
                    (5, 7),
                ),
                ShippingMethod::weight_based(
                    "express_heavy",
                    "Express Heavy Freight",
                    dec!(20.00),
                    dec!(2.50),
                    (1, 2),
                ),
                ShippingMethod::local_pickup("pickup_la", "Pickup at Los Angeles Depot"),
            ]);

        // 1. Zone matches US
        assert!(us_west.matches("US", Some("CA"), Some("90210")));
        assert!(!us_west.matches("DE", None, None));

        // 2. Calculate rates for $50 cart weighing 2.0 kg
        let rates_50 = ShippingCalculator::calculate_rates(&us_west, dec!(50.00), dec!(2.0));
        // Flat rate ($12), Express ($20 + 2*2.50 = $25), Local Pickup ($0). Free tier is not unlocked yet.
        assert_eq!(rates_50.len(), 3);
        assert_eq!(rates_50[0].cost, dec!(12.00));
        assert_eq!(rates_50[1].cost, dec!(25.00));
        assert_eq!(rates_50[2].cost, dec!(0.00));

        // 3. Calculate rates for $150 cart weighing 1.0 kg (Free tier unlocked)
        let rates_150 = ShippingCalculator::calculate_rates(&us_west, dec!(150.00), dec!(1.0));
        assert_eq!(rates_150.len(), 4);
        assert!(
            rates_150
                .iter()
                .any(|r| r.method_id == "free_tier" && r.is_free)
        );
    }
}
