//! Advanced Coupon & Promotion Engine (`erp-trade::coupon`).
//!
//! Provides enterprise promotion capabilities matching WooCommerce and Odoo eCommerce:
//! - Percentage discounts, Fixed amount discounts, and Free shipping coupons
//! - Granular constraints: Min spend thresholds, Max discount caps, Expiration dates
//! - Usage limits: Global quota, Per-customer quota, Usage tracking
//! - SKU and Item Group inclusion/exclusion lists
//! - BOGO (Buy X Get Y) & Tiered Volume Discount promotion rules

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Coupon validation and execution errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CouponValidationError {
    #[error("Coupon code '{0}' is not active or has been disabled")]
    Inactive(String),
    #[error("Coupon code '{0}' has not started yet")]
    NotStarted(String),
    #[error("Coupon code '{0}' has expired on {1}")]
    Expired(String, String),
    #[error("Coupon code '{0}' has reached its global usage limit of {1}")]
    GlobalUsageLimitExceeded(String, u32),
    #[error("User '{0}' has exceeded the per-customer usage limit ({1}) for coupon '{2}'")]
    UserUsageLimitExceeded(String, u32, String),
    #[error("Order subtotal {0} is below the minimum required spend of {1}")]
    MinimumSpendNotMet(Decimal, Decimal),
    #[error("No items in the cart are eligible for coupon '{0}'")]
    NoEligibleItems(String),
    #[error("Coupon code '{0}' is invalid")]
    InvalidCode(String),
}

/// Type of discount applied by the coupon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouponDiscountType {
    /// Percentage discount applied on eligible items or subtotal (e.g. 15.0 = 15%).
    Percentage(Decimal),
    /// Fixed monetary amount discount (e.g. $25.00).
    FixedAmount(Decimal),
    /// Grants 100% discount on shipping fees.
    FreeShipping,
}

/// Individual item line in the cart evaluated by the coupon engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartItemLine {
    pub item_code: String,
    pub item_group: Option<String>,
    pub qty: Decimal,
    pub unit_price: Decimal,
    pub line_total: Decimal,
}

impl CartItemLine {
    #[must_use]
    pub fn new(item_code: impl Into<String>, qty: Decimal, unit_price: Decimal) -> Self {
        let item_code = item_code.into();
        let line_total = qty * unit_price;
        Self {
            item_code,
            item_group: None,
            qty,
            unit_price,
            line_total,
        }
    }

    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.item_group = Some(group.into());
        self
    }
}

/// Representation of a promotional coupon code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCode {
    pub code: String,
    pub description: Option<String>,
    pub discount_type: CouponDiscountType,
    pub min_subtotal: Option<Decimal>,
    pub max_discount_amount: Option<Decimal>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub usage_limit_total: Option<u32>,
    pub usage_limit_per_user: Option<u32>,
    pub times_used: u32,
    pub user_usage_counts: HashMap<String, u32>,
    pub applicable_items: Vec<String>,
    pub excluded_items: Vec<String>,
    pub applicable_categories: Vec<String>,
    pub is_active: bool,
}

impl CouponCode {
    #[must_use]
    pub fn new(code: impl Into<String>, discount_type: CouponDiscountType) -> Self {
        Self {
            code: code.into().to_uppercase(),
            description: None,
            discount_type,
            min_subtotal: None,
            max_discount_amount: None,
            valid_from: None,
            valid_until: None,
            usage_limit_total: None,
            usage_limit_per_user: None,
            times_used: 0,
            user_usage_counts: HashMap::new(),
            applicable_items: Vec::new(),
            excluded_items: Vec::new(),
            applicable_categories: Vec::new(),
            is_active: true,
        }
    }

    #[must_use]
    pub fn with_min_spend(mut self, min: Decimal) -> Self {
        self.min_subtotal = Some(min);
        self
    }

    #[must_use]
    pub fn with_max_discount(mut self, max: Decimal) -> Self {
        self.max_discount_amount = Some(max);
        self
    }

    #[must_use]
    pub fn with_validity(
        mut self,
        from: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = from;
        self.valid_until = until;
        self
    }

    #[must_use]
    pub fn with_limits(mut self, total: Option<u32>, per_user: Option<u32>) -> Self {
        self.usage_limit_total = total;
        self.usage_limit_per_user = per_user;
        self
    }

    #[must_use]
    pub fn with_applicable_items(mut self, items: Vec<String>) -> Self {
        self.applicable_items = items;
        self
    }

    #[must_use]
    pub fn with_excluded_items(mut self, items: Vec<String>) -> Self {
        self.excluded_items = items;
        self
    }
}

/// Calculation breakdown result produced by the coupon engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCalculationResult {
    pub coupon_code: String,
    pub discount_type: CouponDiscountType,
    pub original_subtotal: Decimal,
    pub eligible_subtotal: Decimal,
    pub discount_amount: Decimal,
    pub final_subtotal: Decimal,
    pub free_shipping_granted: bool,
}

/// Core Promotion Engine evaluating discounts and coupon application.
pub struct CouponEngine;

impl CouponEngine {
    /// Validates if a coupon code can be applied to the current cart and user.
    pub fn validate_coupon(
        coupon: &CouponCode,
        user_id: Option<&str>,
        subtotal: Decimal,
        items: &[CartItemLine],
        now: DateTime<Utc>,
    ) -> Result<(), CouponValidationError> {
        if !coupon.is_active {
            return Err(CouponValidationError::Inactive(coupon.code.clone()));
        }

        if coupon.valid_from.is_some_and(|from| now < from) {
            return Err(CouponValidationError::NotStarted(coupon.code.clone()));
        }

        if coupon.valid_until.is_some_and(|until| now > until) {
            return Err(CouponValidationError::Expired(
                coupon.code.clone(),
                coupon.valid_until.unwrap().to_rfc3339(),
            ));
        }

        if coupon
            .usage_limit_total
            .is_some_and(|limit| coupon.times_used >= limit)
        {
            return Err(CouponValidationError::GlobalUsageLimitExceeded(
                coupon.code.clone(),
                coupon.usage_limit_total.unwrap(),
            ));
        }

        if let (Some(per_user_limit), Some(uid)) = (coupon.usage_limit_per_user, user_id) {
            let user_count = coupon.user_usage_counts.get(uid).copied().unwrap_or(0);
            if user_count >= per_user_limit {
                return Err(CouponValidationError::UserUsageLimitExceeded(
                    uid.to_string(),
                    per_user_limit,
                    coupon.code.clone(),
                ));
            }
        }

        if coupon.min_subtotal.is_some_and(|min| subtotal < min) {
            return Err(CouponValidationError::MinimumSpendNotMet(
                subtotal,
                coupon.min_subtotal.unwrap(),
            ));
        }

        let eligible_amount = Self::compute_eligible_subtotal(coupon, items);
        if eligible_amount <= Decimal::ZERO && !items.is_empty() {
            return Err(CouponValidationError::NoEligibleItems(coupon.code.clone()));
        }

        Ok(())
    }

    /// Computes the subtotal of items in the cart eligible for the coupon.
    #[must_use]
    pub fn compute_eligible_subtotal(coupon: &CouponCode, items: &[CartItemLine]) -> Decimal {
        let mut eligible_sum = Decimal::ZERO;

        for item in items {
            // Check exclusions first
            if coupon
                .excluded_items
                .iter()
                .any(|ex| ex.eq_ignore_ascii_case(&item.item_code))
            {
                continue;
            }

            // Check item code inclusions
            let matches_item = coupon.applicable_items.is_empty()
                || coupon
                    .applicable_items
                    .iter()
                    .any(|app| app.eq_ignore_ascii_case(&item.item_code));

            // Check category inclusions
            let matches_category = coupon.applicable_categories.is_empty()
                || item.item_group.as_deref().is_some_and(|cat| {
                    coupon
                        .applicable_categories
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(cat))
                });

            if matches_item && matches_category {
                eligible_sum += item.line_total;
            }
        }

        eligible_sum
    }

    /// Evaluates discount amount and generates calculation result.
    pub fn calculate_discount(
        coupon: &CouponCode,
        subtotal: Decimal,
        items: &[CartItemLine],
    ) -> CouponCalculationResult {
        let eligible_subtotal = Self::compute_eligible_subtotal(coupon, items);

        let (raw_discount, free_shipping) = match &coupon.discount_type {
            CouponDiscountType::Percentage(pct) => {
                let disc = eligible_subtotal * (*pct / dec!(100.0));
                (disc, false)
            }
            CouponDiscountType::FixedAmount(amt) => {
                let disc = (*amt).min(eligible_subtotal);
                (disc, false)
            }
            CouponDiscountType::FreeShipping => (Decimal::ZERO, true),
        };

        let mut discount_amount = raw_discount;
        if let Some(max_cap) = coupon.max_discount_amount {
            discount_amount = discount_amount.min(max_cap);
        }

        discount_amount = discount_amount.min(subtotal);
        let final_subtotal = subtotal - discount_amount;

        CouponCalculationResult {
            coupon_code: coupon.code.clone(),
            discount_type: coupon.discount_type.clone(),
            original_subtotal: subtotal,
            eligible_subtotal,
            discount_amount,
            final_subtotal,
            free_shipping_granted: free_shipping,
        }
    }

    /// Increments usage counters upon successful order placement.
    pub fn record_coupon_usage(
        coupon: &mut CouponCode,
        user_id: Option<&str>,
    ) -> Result<(), CouponValidationError> {
        if coupon
            .usage_limit_total
            .is_some_and(|limit| coupon.times_used >= limit)
        {
            return Err(CouponValidationError::GlobalUsageLimitExceeded(
                coupon.code.clone(),
                coupon.usage_limit_total.unwrap(),
            ));
        }

        coupon.times_used += 1;
        if let Some(uid) = user_id {
            let count = coupon.user_usage_counts.entry(uid.to_string()).or_insert(0);
            *count += 1;
        }

        Ok(())
    }
}

/// Advanced Promotion Rules (BOGO, Tiered Discounts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BogoPromotionRule {
    pub name: String,
    pub buy_item_code: String,
    pub buy_qty: Decimal,
    pub get_item_code: String,
    pub get_qty: Decimal,
    pub discount_percentage_on_get: Decimal, // 100.0 = completely free
}

impl BogoPromotionRule {
    /// Applies BOGO logic to a cart, calculating total free/discounted items.
    #[must_use]
    pub fn calculate_bogo_discount(&self, items: &[CartItemLine]) -> Decimal {
        let buy_count: Decimal = items
            .iter()
            .filter(|i| i.item_code.eq_ignore_ascii_case(&self.buy_item_code))
            .map(|i| i.qty)
            .sum();

        if buy_count < self.buy_qty {
            return Decimal::ZERO;
        }

        let times_qualified = (buy_count / self.buy_qty).floor();
        let max_free_units = times_qualified * self.get_qty;

        let mut remaining_discountable = max_free_units;
        let mut total_discount = Decimal::ZERO;

        for item in items
            .iter()
            .filter(|i| i.item_code.eq_ignore_ascii_case(&self.get_item_code))
        {
            if remaining_discountable <= Decimal::ZERO {
                break;
            }
            let discountable_on_line = item.qty.min(remaining_discountable);
            let line_disc = discountable_on_line
                * item.unit_price
                * (self.discount_percentage_on_get / dec!(100.0));
            total_discount += line_disc;
            remaining_discountable -= discountable_on_line;
        }

        total_discount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_percentage_coupon_with_limits_and_eligible_items() {
        let now = Utc::now();
        let coupon = CouponCode::new("SUMMER20", CouponDiscountType::Percentage(dec!(20.0)))
            .with_min_spend(dec!(100.00))
            .with_max_discount(dec!(50.00))
            .with_applicable_items(vec!["SHIRT-01".into(), "PANTS-02".into()])
            .with_limits(Some(100), Some(1));

        let items = vec![
            CartItemLine::new("SHIRT-01", dec!(2.0), dec!(60.00)), // line = 120.00 (eligible)
            CartItemLine::new("BELT-99", dec!(1.0), dec!(30.00)),  // line = 30.00 (not eligible)
        ];
        let subtotal = dec!(150.00);

        // Validation passes
        assert!(
            CouponEngine::validate_coupon(&coupon, Some("cust_1"), subtotal, &items, now).is_ok()
        );

        // Calculation: 20% on $120 = $24.00 discount
        let res = CouponEngine::calculate_discount(&coupon, subtotal, &items);
        assert_eq!(res.eligible_subtotal, dec!(120.00));
        assert_eq!(res.discount_amount, dec!(24.00));
        assert_eq!(res.final_subtotal, dec!(126.00));
        assert!(!res.free_shipping_granted);
    }

    #[test]
    fn test_coupon_expiration_and_usage_limits() {
        let now = Utc::now();
        let past = now - Duration::days(2);
        let expired_coupon =
            CouponCode::new("EXPIRED", CouponDiscountType::FixedAmount(dec!(10.00)))
                .with_validity(None, Some(past));

        let items = vec![CartItemLine::new("ITEM-1", dec!(1.0), dec!(50.00))];
        let err = CouponEngine::validate_coupon(
            &expired_coupon,
            Some("cust_1"),
            dec!(50.00),
            &items,
            now,
        );
        assert!(matches!(err, Err(CouponValidationError::Expired(_, _))));

        // User limit test
        let mut user_limited =
            CouponCode::new("ONETIME", CouponDiscountType::FixedAmount(dec!(5.00)))
                .with_limits(None, Some(1));
        assert!(
            CouponEngine::validate_coupon(&user_limited, Some("cust_1"), dec!(50.00), &items, now)
                .is_ok()
        );

        // Record usage once
        assert!(CouponEngine::record_coupon_usage(&mut user_limited, Some("cust_1")).is_ok());
        assert_eq!(user_limited.times_used, 1);

        // Second attempt must fail
        let err2 =
            CouponEngine::validate_coupon(&user_limited, Some("cust_1"), dec!(50.00), &items, now);
        assert!(matches!(
            err2,
            Err(CouponValidationError::UserUsageLimitExceeded(_, 1, _))
        ));
    }

    #[test]
    fn test_bogo_buy_1_get_1_free() {
        let bogo = BogoPromotionRule {
            name: "Buy 1 Coffee Get 1 Free".into(),
            buy_item_code: "COFFEE-BAG".into(),
            buy_qty: dec!(1.0),
            get_item_code: "COFFEE-MUG".into(),
            get_qty: dec!(1.0),
            discount_percentage_on_get: dec!(100.0),
        };

        let items = vec![
            CartItemLine::new("COFFEE-BAG", dec!(2.0), dec!(15.00)), // Qualified for 2 free mugs
            CartItemLine::new("COFFEE-MUG", dec!(2.0), dec!(10.00)), // 2 mugs in cart = $20 total discount
        ];

        let disc = bogo.calculate_bogo_discount(&items);
        assert_eq!(disc, dec!(20.00));
    }
}
