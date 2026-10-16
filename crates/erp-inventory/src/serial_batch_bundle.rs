//! Serial and Batch Bundle Engine (`erp-inventory::serial_batch_bundle`).
//!
//! Mirrors Frappe/ERPNext v15+ unified Serial and Batch Bundle architecture (`tabSerial and Batch Bundle`),
//! pairing discrete serial numbers and manufacturing batch lots to inventory vouchers
//! (Purchase Receipt, Delivery Note, Stock Entry).

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Serial and batch bundle errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BundleError {
    #[error("Duplicate serial number '{0}' detected in bundle")]
    DuplicateSerialNo(CompactString),
    #[error(
        "Sum of entry quantities ({sum_entries}) does not match bundle total quantity ({total_qty})"
    )]
    QuantityMismatch {
        sum_entries: Decimal,
        total_qty: Decimal,
    },
    #[error("Empty entries list in Serial and Batch Bundle")]
    EmptyBundle,
}

/// Type of stock movement represented by this bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundleType {
    Inward,
    Outward,
}

/// An individual line item in the `Serial and Batch Entry` child table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerialBatchEntry {
    pub serial_no: Option<CompactString>,
    pub batch_no: Option<CompactString>,
    pub qty: Decimal,
    pub incoming_rate: Decimal,
}

/// Unified Serial and Batch Bundle container document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerialBatchBundle {
    pub name: CompactString,
    pub voucher_type: CompactString, // "Purchase Receipt", "Delivery Note", "Stock Entry"
    pub voucher_no: CompactString,
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub bundle_type: BundleType,
    pub total_qty: Decimal,
    pub avg_rate: Decimal,
    pub total_amount: Decimal,
    pub entries: Vec<SerialBatchEntry>,
}

impl SerialBatchBundle {
    /// Validates internal consistency, duplicate serials, and total quantity matching.
    pub fn validate(&self) -> Result<(), BundleError> {
        if self.entries.is_empty() {
            return Err(BundleError::EmptyBundle);
        }

        let mut serial_set = HashSet::new();
        let mut sum_qty = Decimal::ZERO;

        for entry in &self.entries {
            sum_qty += entry.qty;

            if let Some(ref sn) = entry.serial_no
                && !serial_set.insert(sn.clone())
            {
                return Err(BundleError::DuplicateSerialNo(sn.clone()));
            }
        }

        if sum_qty != self.total_qty {
            return Err(BundleError::QuantityMismatch {
                sum_entries: sum_qty,
                total_qty: self.total_qty,
            });
        }

        Ok(())
    }

    /// Computes weighted average rate and total valuation amount from entries.
    #[must_use]
    pub fn calculate_totals(&self) -> (Decimal, Decimal) {
        let mut total_amt = Decimal::ZERO;
        let mut total_q = Decimal::ZERO;

        for entry in &self.entries {
            total_amt += entry.qty * entry.incoming_rate;
            total_q += entry.qty;
        }

        let avg = if !total_q.is_zero() {
            total_amt / total_q
        } else {
            Decimal::ZERO
        };

        (avg, total_amt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_serial_batch_bundle_validation_success() {
        let bundle = SerialBatchBundle {
            name: "SBB-001".into(),
            voucher_type: "Purchase Receipt".into(),
            voucher_no: "PR-2026-001".into(),
            item_code: "SMARTPHONE-PRO".into(),
            warehouse: "Finished Goods - CORP".into(),
            bundle_type: BundleType::Inward,
            total_qty: dec!(3.0),
            avg_rate: dec!(800.0),
            total_amount: dec!(2400.0),
            entries: vec![
                SerialBatchEntry {
                    serial_no: Some("SN-1001".into()),
                    batch_no: Some("B-OCT-2026".into()),
                    qty: dec!(1.0),
                    incoming_rate: dec!(800.0),
                },
                SerialBatchEntry {
                    serial_no: Some("SN-1002".into()),
                    batch_no: Some("B-OCT-2026".into()),
                    qty: dec!(1.0),
                    incoming_rate: dec!(800.0),
                },
                SerialBatchEntry {
                    serial_no: Some("SN-1003".into()),
                    batch_no: Some("B-OCT-2026".into()),
                    qty: dec!(1.0),
                    incoming_rate: dec!(800.0),
                },
            ],
        };

        assert!(bundle.validate().is_ok());
        let (avg, total) = bundle.calculate_totals();
        assert_eq!(avg, dec!(800.0));
        assert_eq!(total, dec!(2400.0));
    }

    #[test]
    fn test_duplicate_serial_rejection() {
        let bundle = SerialBatchBundle {
            name: "SBB-002".into(),
            voucher_type: "Delivery Note".into(),
            voucher_no: "DN-001".into(),
            item_code: "ROUTER-5G".into(),
            warehouse: "Finished Goods - CORP".into(),
            bundle_type: BundleType::Outward,
            total_qty: dec!(2.0),
            avg_rate: dec!(150.0),
            total_amount: dec!(300.0),
            entries: vec![
                SerialBatchEntry {
                    serial_no: Some("SN-DUP".into()),
                    batch_no: None,
                    qty: dec!(1.0),
                    incoming_rate: dec!(150.0),
                },
                SerialBatchEntry {
                    serial_no: Some("SN-DUP".into()), // Duplicate!
                    batch_no: None,
                    qty: dec!(1.0),
                    incoming_rate: dec!(150.0),
                },
            ],
        };

        assert_eq!(
            bundle.validate(),
            Err(BundleError::DuplicateSerialNo("SN-DUP".into()))
        );
    }
}
