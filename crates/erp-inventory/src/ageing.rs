use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configurable aging day bucket range (e.g. 0..=30, 31..=60).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgeingBucket {
    /// Bucket display label (e.g. "0-30 Days").
    pub label: String,
    /// Minimum elapsed days inclusive.
    pub min_days: i64,
    /// Maximum elapsed days inclusive (None for infinity / over X days).
    pub max_days: Option<i64>,
}

impl AgeingBucket {
    /// Standard ERPNext 30-day interval buckets.
    pub fn default_buckets() -> Vec<Self> {
        vec![
            Self {
                label: "0 - 30 Days".into(),
                min_days: 0,
                max_days: Some(30),
            },
            Self {
                label: "31 - 60 Days".into(),
                min_days: 31,
                max_days: Some(60),
            },
            Self {
                label: "61 - 90 Days".into(),
                min_days: 61,
                max_days: Some(90),
            },
            Self {
                label: "91 - 120 Days".into(),
                min_days: 91,
                max_days: Some(120),
            },
            Self {
                label: "120+ Days".into(),
                min_days: 121,
                max_days: None,
            },
        ]
    }

    /// Checks if a given age in days falls into this bucket.
    pub fn contains_days(&self, days: i64) -> bool {
        if days < self.min_days {
            return false;
        }
        match self.max_days {
            Some(max) => days <= max,
            None => true,
        }
    }
}

/// Inbound stock receipt entry with timestamp for FIFO age tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockReceiptLayer {
    /// Date receipt was posted.
    pub posting_date: NaiveDate,
    /// Quantity received and remaining.
    pub qty: Decimal,
    /// Valuation rate.
    pub rate: Decimal,
    /// Optional batch number.
    pub batch_no: Option<String>,
    /// Optional expiry date.
    pub expiry_date: Option<NaiveDate>,
}

/// Calculated stock ageing breakdown for an item in a warehouse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockAgeingSummary {
    /// Target item code.
    pub item_code: String,
    /// Warehouse location.
    pub warehouse: String,
    /// Total remaining quantity on hand.
    pub total_qty: Decimal,
    /// Total remaining stock valuation.
    pub total_value: Decimal,
    /// Earliest batch expiry date among remaining stock layers.
    pub earliest_expiry: Option<NaiveDate>,
    /// Average age of inventory in days.
    pub average_age_days: Decimal,
    /// Quantity and monetary value mapped to each configured aging bucket.
    pub bucket_values: Vec<(String, Decimal, Decimal)>,
}

/// Streaming Stock Ageing calculation engine.
pub struct StockAgeingEngine;

impl StockAgeingEngine {
    /// Evaluates receipt layers against disbursements using FIFO to compute ageing distribution as of `as_of_date`.
    pub fn compute_ageing(
        item_code: &str,
        warehouse: &str,
        receipts: &[StockReceiptLayer],
        disbursed_qty: Decimal,
        as_of_date: NaiveDate,
        buckets: &[AgeingBucket],
    ) -> StockAgeingSummary {
        let mut remaining_to_disburse = disbursed_qty;
        let mut active_layers = Vec::new();

        for receipt in receipts {
            if remaining_to_disburse >= receipt.qty {
                remaining_to_disburse -= receipt.qty;
                continue;
            }

            let available_qty = receipt.qty - remaining_to_disburse;
            remaining_to_disburse = Decimal::ZERO;

            active_layers.push(StockReceiptLayer {
                posting_date: receipt.posting_date,
                qty: available_qty,
                rate: receipt.rate,
                batch_no: receipt.batch_no.clone(),
                expiry_date: receipt.expiry_date,
            });
        }

        let total_qty: Decimal = active_layers.iter().map(|l| l.qty).sum();
        let total_value: Decimal = active_layers.iter().map(|l| l.qty * l.rate).sum();

        let earliest_expiry = active_layers
            .iter()
            .filter_map(|l| l.expiry_date)
            .min();

        // Calculate bucket breakdowns and weighted average age
        let mut bucket_acc: Vec<(String, Decimal, Decimal)> = buckets
            .iter()
            .map(|b| (b.label.clone(), Decimal::ZERO, Decimal::ZERO))
            .collect();

        let mut total_age_weighted_sum = Decimal::ZERO;

        for layer in &active_layers {
            let days_old = (as_of_date - layer.posting_date).num_days().max(0);
            let layer_value = layer.qty * layer.rate;
            total_age_weighted_sum += layer.qty * Decimal::from(days_old);

            for (i, bucket) in buckets.iter().enumerate() {
                if bucket.contains_days(days_old) {
                    bucket_acc[i].1 += layer.qty;
                    bucket_acc[i].2 += layer_value;
                    break;
                }
            }
        }

        let average_age_days = if total_qty.is_zero() {
            Decimal::ZERO
        } else {
            total_age_weighted_sum / total_qty
        };

        StockAgeingSummary {
            item_code: item_code.to_string(),
            warehouse: warehouse.to_string(),
            total_qty,
            total_value,
            earliest_expiry,
            average_age_days,
            bucket_values: bucket_acc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_stock_ageing_calculation() {
        let buckets = AgeingBucket::default_buckets();
        let as_of = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();

        let receipts = vec![
            StockReceiptLayer {
                posting_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), // ~152 days old (>120)
                qty: dec!(100),
                rate: dec!(10),
                batch_no: Some("BATCH-MAY".into()),
                expiry_date: Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            },
            StockReceiptLayer {
                posting_date: NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(), // ~46 days old (31-60)
                qty: dec!(50),
                rate: dec!(12),
                batch_no: Some("BATCH-AUG".into()),
                expiry_date: Some(NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()),
            },
            StockReceiptLayer {
                posting_date: NaiveDate::from_ymd_opt(2026, 9, 20).unwrap(), // 10 days old (0-30)
                qty: dec!(20),
                rate: dec!(15),
                batch_no: None,
                expiry_date: None,
            },
        ];

        // Consume 80 units of the oldest layer (leaving 20 units in oldest layer)
        let summary = StockAgeingEngine::compute_ageing(
            "RAW-CHIP-X",
            "Stores - ACME",
            &receipts,
            dec!(80),
            as_of,
            &buckets,
        );

        assert_eq!(summary.total_qty, dec!(90)); // 20 + 50 + 20
        assert_eq!(
            summary.earliest_expiry,
            Some(NaiveDate::from_ymd_opt(2026, 10, 15).unwrap())
        );

        // Check bucket values:
        // 0-30: 20 units @ 15 = 300
        assert_eq!(summary.bucket_values[0].1, dec!(20));
        assert_eq!(summary.bucket_values[0].2, dec!(300));

        // 31-60: 50 units @ 12 = 600
        assert_eq!(summary.bucket_values[1].1, dec!(50));
        assert_eq!(summary.bucket_values[1].2, dec!(600));

        // 120+: 20 units @ 10 = 200
        assert_eq!(summary.bucket_values[4].1, dec!(20));
        assert_eq!(summary.bucket_values[4].2, dec!(200));
    }
}
