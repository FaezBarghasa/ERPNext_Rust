//! Subscription & SaaS Recurring Billing Engine.
//!
//! Supports Flat-Rate, Per-Seat, Tiered (Volume & Graduated), and Metered usage pricing models
//! with arbitrary-precision decimal arithmetic (`rust_decimal::Decimal`).

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::errors::SoftwareBillingError;

/// Volume or graduated tier boundary definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeTier {
    /// Upper limit of units for this tier (`None` represents unbounded / infinity).
    pub up_to: Option<Decimal>,
    /// Unit price applied in this tier.
    pub unit_price: Decimal,
}

/// Software subscription pricing model variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanPricingModel {
    /// Fixed recurrent fee regardless of usage or seat count.
    FlatRate { price: Decimal },
    /// Fixed fee per licensed active seat.
    PerSeat {
        price_per_seat: Decimal,
        min_seats: usize,
    },
    /// Volume tiering: all units priced at the unit price of the highest tier bracket reached.
    TieredVolume { tiers: Vec<VolumeTier> },
    /// Graduated tiering: units priced incrementally across tier brackets.
    TieredGraduated { tiers: Vec<VolumeTier> },
    /// Pay-as-you-go metered consumption.
    MeteredUsage {
        metric_name: String,
        price_per_unit: Decimal,
    },
}

/// Billing recurring interval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BillingInterval {
    Monthly,
    Quarterly,
    Annual,
    CustomDays(u32),
}

impl BillingInterval {
    /// Computes the subsequent period end date given a start date.
    #[must_use]
    pub fn next_period_end(&self, start: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            BillingInterval::Monthly => start + Duration::days(30),
            BillingInterval::Quarterly => start + Duration::days(90),
            BillingInterval::Annual => start + Duration::days(365),
            BillingInterval::CustomDays(days) => start + Duration::days(i64::from(*days)),
        }
    }
}

/// Subscription plan template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub pricing_model: PlanPricingModel,
    pub interval: BillingInterval,
    pub currency: String,
    pub deferred_revenue_account: String,
    pub recognized_revenue_account: String,
}

/// Operational state of a subscription.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Canceled,
    Expired,
}

/// Active customer subscription agreement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subscription {
    pub id: String,
    pub customer_id: String,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub seat_count: usize,
    pub discount_percent: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Ingested metered consumption event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeteredUsageEvent {
    pub id: String,
    pub customer_id: String,
    pub subscription_id: String,
    pub metric_name: String,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
    pub idempotency_key: String,
}

/// Collector for metered usage events with deduplication.
#[derive(Debug, Default)]
pub struct UsageCollector {
    processed_keys: HashSet<String>,
    events: Vec<MeteredUsageEvent>,
}

impl UsageCollector {
    /// Instantiates an empty usage collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a usage event with strict idempotency verification.
    pub fn record(&mut self, event: MeteredUsageEvent) -> Result<(), SoftwareBillingError> {
        if !self.processed_keys.insert(event.idempotency_key.clone()) {
            return Err(SoftwareBillingError::UsageIdempotencyViolation(
                event.idempotency_key,
            ));
        }
        self.events.push(event);
        Ok(())
    }

    /// Aggregates total recorded usage quantity for a specific subscription and metric within a time range.
    #[must_use]
    pub fn aggregate_usage(
        &self,
        subscription_id: &str,
        metric_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Decimal {
        self.events
            .iter()
            .filter(|e| {
                e.subscription_id == subscription_id
                    && e.metric_name == metric_name
                    && e.timestamp >= start
                    && e.timestamp < end
            })
            .fold(Decimal::ZERO, |acc, e| acc + e.quantity)
    }
}

/// Invoice line item description and calculated amount.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub amount: Decimal,
    pub account: String,
}

/// Calculated subscription invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionInvoice {
    pub invoice_id: String,
    pub customer_id: String,
    pub subscription_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub credit_applied: Decimal,
    pub tax_rate_percent: Decimal,
    pub tax_amount: Decimal,
    pub total_due: Decimal,
    pub line_items: Vec<InvoiceLineItem>,
    pub issued_at: DateTime<Utc>,
}

/// Calculator for Volume Tiering: entire quantity is billed at the unit rate of the matched tier.
pub fn calculate_volume_tier_price(
    quantity: Decimal,
    tiers: &[VolumeTier],
) -> Result<Decimal, SoftwareBillingError> {
    if tiers.is_empty() {
        return Err(SoftwareBillingError::InvalidTierConfiguration(
            "Tiers list cannot be empty".into(),
        ));
    }

    if quantity <= Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    // Find the tier that accommodates the total quantity
    for tier in tiers {
        if let Some(limit) = tier.up_to {
            if quantity <= limit {
                return Ok(quantity * tier.unit_price);
            }
        } else {
            // Unbounded top tier
            return Ok(quantity * tier.unit_price);
        }
    }

    // If beyond all bounded tiers without an unbounded tier
    let last_price = tiers
        .last()
        .map(|t| t.unit_price)
        .ok_or_else(|| SoftwareBillingError::InvalidTierConfiguration("Missing tier".into()))?;
    Ok(quantity * last_price)
}

/// Calculator for Graduated Tiering: units within each tier bracket are billed at that bracket's unit rate.
pub fn calculate_graduated_tier_price(
    quantity: Decimal,
    tiers: &[VolumeTier],
) -> Result<Decimal, SoftwareBillingError> {
    if tiers.is_empty() {
        return Err(SoftwareBillingError::InvalidTierConfiguration(
            "Tiers list cannot be empty".into(),
        ));
    }

    if quantity <= Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let mut remaining = quantity;
    let mut total = Decimal::ZERO;
    let mut previous_limit = Decimal::ZERO;

    for tier in tiers {
        if remaining <= Decimal::ZERO {
            break;
        }

        match tier.up_to {
            Some(limit) => {
                if limit <= previous_limit {
                    return Err(SoftwareBillingError::InvalidTierConfiguration(
                        "Tier limits must be strictly monotonically increasing".into(),
                    ));
                }
                let tier_capacity = limit - previous_limit;
                let units_in_tier = remaining.min(tier_capacity);
                total += units_in_tier * tier.unit_price;
                remaining -= units_in_tier;
                previous_limit = limit;
            }
            None => {
                // Unbounded last tier
                total += remaining * tier.unit_price;
                remaining = Decimal::ZERO;
            }
        }
    }

    if remaining > Decimal::ZERO {
        return Err(SoftwareBillingError::InvalidTierConfiguration(
            "Quantity exceeds highest defined tier limit without an unbounded tier".into(),
        ));
    }

    Ok(total)
}

/// Generates a complete `SubscriptionInvoice` for a billing period.
pub fn generate_subscription_invoice(
    invoice_id: String,
    subscription: &Subscription,
    plan: &SubscriptionPlan,
    usage_collector: Option<&UsageCollector>,
    available_credits: Decimal,
    tax_rate_percent: Decimal,
    now: DateTime<Utc>,
) -> Result<SubscriptionInvoice, SoftwareBillingError> {
    if subscription.status != SubscriptionStatus::Active {
        return Err(SoftwareBillingError::InvalidContractTerms(format!(
            "Cannot bill subscription with status {:?}",
            subscription.status
        )));
    }

    let mut line_items = Vec::new();

    match &plan.pricing_model {
        PlanPricingModel::FlatRate { price } => {
            line_items.push(InvoiceLineItem {
                description: format!("{} - Base Recurring Fee", plan.name),
                quantity: Decimal::ONE,
                unit_price: *price,
                amount: *price,
                account: plan.deferred_revenue_account.clone(),
            });
        }
        PlanPricingModel::PerSeat {
            price_per_seat,
            min_seats,
        } => {
            let billable_seats = subscription.seat_count.max(*min_seats);
            let qty = Decimal::from(billable_seats);
            let amount = qty * *price_per_seat;
            line_items.push(InvoiceLineItem {
                description: format!("{} - {} User Seats", plan.name, billable_seats),
                quantity: qty,
                unit_price: *price_per_seat,
                amount,
                account: plan.deferred_revenue_account.clone(),
            });
        }
        PlanPricingModel::TieredVolume { tiers } => {
            let qty = Decimal::from(subscription.seat_count);
            let amount = calculate_volume_tier_price(qty, tiers)?;
            let unit_price = if qty > Decimal::ZERO {
                amount / qty
            } else {
                Decimal::ZERO
            };
            line_items.push(InvoiceLineItem {
                description: format!("{} - Tiered Volume (Qty {})", plan.name, qty),
                quantity: qty,
                unit_price,
                amount,
                account: plan.deferred_revenue_account.clone(),
            });
        }
        PlanPricingModel::TieredGraduated { tiers } => {
            let qty = Decimal::from(subscription.seat_count);
            let amount = calculate_graduated_tier_price(qty, tiers)?;
            let unit_price = if qty > Decimal::ZERO {
                amount / qty
            } else {
                Decimal::ZERO
            };
            line_items.push(InvoiceLineItem {
                description: format!("{} - Graduated Tier (Qty {})", plan.name, qty),
                quantity: qty,
                unit_price,
                amount,
                account: plan.deferred_revenue_account.clone(),
            });
        }
        PlanPricingModel::MeteredUsage {
            metric_name,
            price_per_unit,
        } => {
            let qty = if let Some(collector) = usage_collector {
                collector.aggregate_usage(
                    &subscription.id,
                    metric_name,
                    subscription.current_period_start,
                    subscription.current_period_end,
                )
            } else {
                Decimal::ZERO
            };
            let amount = qty * *price_per_unit;
            line_items.push(InvoiceLineItem {
                description: format!("{} - Consumption: {} ({})", plan.name, metric_name, qty),
                quantity: qty,
                unit_price: *price_per_unit,
                amount,
                account: plan.recognized_revenue_account.clone(),
            });
        }
    }

    if line_items.is_empty() {
        return Err(SoftwareBillingError::EmptyInvoice);
    }

    let subtotal = line_items
        .iter()
        .fold(Decimal::ZERO, |acc, item| acc + item.amount);

    let discount_amount = if subscription.discount_percent > Decimal::ZERO {
        (subtotal * subscription.discount_percent) / dec!(100)
    } else {
        Decimal::ZERO
    };

    let discounted_subtotal = (subtotal - discount_amount).max(Decimal::ZERO);

    // Apply available SLA/overpayment credits
    let credit_applied = available_credits.min(discounted_subtotal);
    let post_credit_taxable = discounted_subtotal - credit_applied;

    let tax_amount = if tax_rate_percent > Decimal::ZERO {
        (post_credit_taxable * tax_rate_percent) / dec!(100)
    } else {
        Decimal::ZERO
    };

    let total_due = post_credit_taxable + tax_amount;

    Ok(SubscriptionInvoice {
        invoice_id,
        customer_id: subscription.customer_id.clone(),
        subscription_id: subscription.id.clone(),
        period_start: subscription.current_period_start,
        period_end: subscription.current_period_end,
        subtotal,
        discount_amount,
        credit_applied,
        tax_rate_percent,
        tax_amount,
        total_due,
        line_items,
        issued_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_volume_tiered_pricing() {
        let tiers = vec![
            VolumeTier {
                up_to: Some(dec!(10)),
                unit_price: dec!(20),
            },
            VolumeTier {
                up_to: Some(dec!(50)),
                unit_price: dec!(15),
            },
            VolumeTier {
                up_to: None,
                unit_price: dec!(10),
            },
        ];

        // 5 units @ $20 = $100
        assert_eq!(
            calculate_volume_tier_price(dec!(5), &tiers).unwrap(),
            dec!(100)
        );
        // 20 units @ $15 = $300
        assert_eq!(
            calculate_volume_tier_price(dec!(20), &tiers).unwrap(),
            dec!(300)
        );
        // 100 units @ $10 = $1000
        assert_eq!(
            calculate_volume_tier_price(dec!(100), &tiers).unwrap(),
            dec!(1000)
        );
    }

    #[test]
    fn test_graduated_tiered_pricing() {
        let tiers = vec![
            VolumeTier {
                up_to: Some(dec!(10)),
                unit_price: dec!(20),
            },
            VolumeTier {
                up_to: Some(dec!(50)),
                unit_price: dec!(15),
            },
            VolumeTier {
                up_to: None,
                unit_price: dec!(10),
            },
        ];

        // 5 units: 5 * 20 = $100
        assert_eq!(
            calculate_graduated_tier_price(dec!(5), &tiers).unwrap(),
            dec!(100)
        );
        // 20 units: 10 * 20 + 10 * 15 = 200 + 150 = $350
        assert_eq!(
            calculate_graduated_tier_price(dec!(20), &tiers).unwrap(),
            dec!(350)
        );
        // 60 units: 10 * 20 + 40 * 15 + 10 * 10 = 200 + 600 + 100 = $900
        assert_eq!(
            calculate_graduated_tier_price(dec!(60), &tiers).unwrap(),
            dec!(900)
        );
    }

    #[test]
    fn test_usage_collector_idempotency() {
        let mut collector = UsageCollector::new();
        let now = Utc::now();

        let evt1 = MeteredUsageEvent {
            id: "evt-1".into(),
            customer_id: "cust-1".into(),
            subscription_id: "sub-1".into(),
            metric_name: "api_calls".into(),
            quantity: dec!(500),
            timestamp: now,
            idempotency_key: "key-100".into(),
        };

        assert!(collector.record(evt1.clone()).is_ok());
        // Duplicate key must error
        assert!(matches!(
            collector.record(evt1),
            Err(SoftwareBillingError::UsageIdempotencyViolation(_))
        ));

        let total = collector.aggregate_usage(
            "sub-1",
            "api_calls",
            now - Duration::hours(1),
            now + Duration::hours(1),
        );
        assert_eq!(total, dec!(500));
    }

    #[test]
    fn test_invoice_generation_with_discount_and_credits() {
        let now = Utc::now();
        let plan = SubscriptionPlan {
            id: "plan-pro".into(),
            name: "Enterprise Cloud".into(),
            pricing_model: PlanPricingModel::PerSeat {
                price_per_seat: dec!(50),
                min_seats: 5,
            },
            interval: BillingInterval::Monthly,
            currency: "USD".into(),
            deferred_revenue_account: "2100-Deferred-Rev".into(),
            recognized_revenue_account: "4000-SaaS-Rev".into(),
        };

        let sub = Subscription {
            id: "sub-10".into(),
            customer_id: "cust-99".into(),
            plan_id: "plan-pro".into(),
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: now + Duration::days(30),
            seat_count: 10,
            discount_percent: dec!(10), // 10% discount
            created_at: now,
        };

        // 10 seats * $50 = $500 subtotal
        // Discount 10% = $50 -> $450
        // Credits applied = $100 -> $350
        // Tax 10% on $350 = $35
        // Total due = $385
        let inv = generate_subscription_invoice(
            "INV-2026-001".into(),
            &sub,
            &plan,
            None,
            dec!(100), // available credit
            dec!(10),  // 10% tax
            now,
        )
        .unwrap();

        assert_eq!(inv.subtotal, dec!(500));
        assert_eq!(inv.discount_amount, dec!(50));
        assert_eq!(inv.credit_applied, dec!(100));
        assert_eq!(inv.tax_amount, dec!(35));
        assert_eq!(inv.total_due, dec!(385));
    }
}
