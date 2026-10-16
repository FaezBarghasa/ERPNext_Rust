//! Accounts Receivable & Accounts Payable Aging Report Engine (`erp-accounting::aging`).
//!
//! Categorizes open invoice balances across aging intervals (0-30, 31-60, 61-90, 91-120, 120+ days)
//! based on document due dates relative to an evaluation cutoff date.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aging bucket breakdown for an individual party.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgingBucket {
    pub range_0_30: Decimal,
    pub range_31_60: Decimal,
    pub range_61_90: Decimal,
    pub range_91_120: Decimal,
    pub range_121_above: Decimal,
    pub total_outstanding: Decimal,
}

/// An open invoice item considered in the aging analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgingInvoice {
    pub voucher_no: CompactString,
    pub party_type: CompactString, // "Customer" or "Supplier"
    pub party: CompactString,
    pub party_name: CompactString,
    pub posting_date: NaiveDate,
    pub due_date: NaiveDate,
    pub outstanding_amount: Decimal,
}

/// Aggregated aging summary row for a customer or supplier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyAgingRow {
    pub party_type: CompactString,
    pub party: CompactString,
    pub party_name: CompactString,
    pub bucket: AgingBucket,
}

/// Aging Report Calculation Engine.
pub struct AgingReportEngine;

impl AgingReportEngine {
    /// Generates aging bucket matrix for a list of open invoices as of a specific date.
    #[must_use]
    pub fn calculate_aging(invoices: &[AgingInvoice], as_of_date: NaiveDate) -> Vec<PartyAgingRow> {
        let mut party_map: HashMap<(CompactString, CompactString, CompactString), AgingBucket> =
            HashMap::new();

        for inv in invoices {
            if inv.outstanding_amount <= Decimal::ZERO {
                continue;
            }

            let days_overdue = (as_of_date - inv.due_date).num_days();

            let entry = party_map
                .entry((
                    inv.party_type.clone(),
                    inv.party.clone(),
                    inv.party_name.clone(),
                ))
                .or_default();

            entry.total_outstanding += inv.outstanding_amount;

            if days_overdue <= 30 {
                entry.range_0_30 += inv.outstanding_amount;
            } else if days_overdue <= 60 {
                entry.range_31_60 += inv.outstanding_amount;
            } else if days_overdue <= 90 {
                entry.range_61_90 += inv.outstanding_amount;
            } else if days_overdue <= 120 {
                entry.range_91_120 += inv.outstanding_amount;
            } else {
                entry.range_121_above += inv.outstanding_amount;
            }
        }

        let mut results: Vec<PartyAgingRow> = party_map
            .into_iter()
            .map(|((party_type, party, party_name), bucket)| PartyAgingRow {
                party_type,
                party,
                party_name,
                bucket,
            })
            .collect();

        // Sort by total outstanding descending
        results.sort_by_key(|a| std::cmp::Reverse(a.bucket.total_outstanding));

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_aging_interval_classification() {
        let as_of_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        let invoices = vec![
            AgingInvoice {
                voucher_no: "ACC-SINV-001".into(),
                party_type: "Customer".into(),
                party: "CUST-001".into(),
                party_name: "Apex Logistics".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
                due_date: NaiveDate::from_ymd_opt(2026, 10, 5).unwrap(), // 11 days overdue -> 0-30
                outstanding_amount: dec!(500.00),
            },
            AgingInvoice {
                voucher_no: "ACC-SINV-002".into(),
                party_type: "Customer".into(),
                party: "CUST-001".into(),
                party_name: "Apex Logistics".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
                due_date: NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(), // 62 days overdue -> 61-90
                outstanding_amount: dec!(1200.00),
            },
            AgingInvoice {
                voucher_no: "ACC-SINV-003".into(),
                party_type: "Customer".into(),
                party: "CUST-002".into(),
                party_name: "Zenith Corp".into(),
                posting_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
                due_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(), // 154 days overdue -> 121+
                outstanding_amount: dec!(3000.00),
            },
        ];

        let report = AgingReportEngine::calculate_aging(&invoices, as_of_date);

        assert_eq!(report.len(), 2);

        // Zenith Corp has highest outstanding ($3,000)
        assert_eq!(report[0].party, "CUST-002");
        assert_eq!(report[0].bucket.total_outstanding, dec!(3000.00));
        assert_eq!(report[0].bucket.range_121_above, dec!(3000.00));

        // Apex Logistics has $1,700 total ($500 in 0-30, $1200 in 61-90)
        assert_eq!(report[1].party, "CUST-001");
        assert_eq!(report[1].bucket.total_outstanding, dec!(1700.00));
        assert_eq!(report[1].bucket.range_0_30, dec!(500.00));
        assert_eq!(report[1].bucket.range_61_90, dec!(1200.00));
    }
}
