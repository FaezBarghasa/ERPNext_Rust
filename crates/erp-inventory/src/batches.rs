use crate::fifo::InventoryError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Operational status of a serialized asset unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerialStatus {
    /// In warehouse on-hand.
    Active,
    /// Dispatched / Delivered to customer.
    Delivered,
    /// Expired or decommissioned.
    Expired,
    /// Under maintenance / repair.
    Maintenance,
}

/// Serial Number Registry record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerialNo {
    /// Unique serial string.
    pub serial_no: String,
    /// Associated item code.
    pub item_code: String,
    /// Current warehouse location.
    pub warehouse: String,
    /// Current operational status.
    pub status: SerialStatus,
}

impl SerialNo {
    /// Validates serial number location prior to dispatch.
    pub fn validate_dispatch_from(&self, from_warehouse: &str) -> Result<(), InventoryError> {
        if self.warehouse != from_warehouse {
            return Err(InventoryError::SerialNotInWarehouse {
                serial_no: self.serial_no.clone(),
                expected_warehouse: from_warehouse.to_string(),
                actual_warehouse: self.warehouse.clone(),
            });
        }
        Ok(())
    }
}

/// Batch tracking record with expiration management.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Batch {
    /// Unique batch identifier.
    pub batch_id: String,
    /// Associated item code.
    pub item_code: String,
    /// Manufacturing date.
    pub mfg_date: NaiveDate,
    /// Expiry date (if perishable).
    pub expiry_date: Option<NaiveDate>,
    /// Remaining batch quantity in stock.
    pub remaining_qty: Decimal,
}

impl Batch {
    /// Validates whether the batch is active and unexpired as of a given posting date.
    pub fn validate_usable(&self, as_of: NaiveDate) -> Result<(), InventoryError> {
        if let Some(exp) = self.expiry_date
            && as_of > exp
        {
            return Err(InventoryError::BatchExpired {
                batch_id: self.batch_id.clone(),
                expiry_date: exp,
            });
        }

        Ok(())
    }

    /// Selects batches using First-Expired-First-Out (FEFO) or FIFO (mfg_date) order
    /// to fulfill requested item quantity, rejecting expired batches.
    pub fn select_fifo_batches(
        available_batches: &mut [Batch],
        required_qty: Decimal,
        as_of: NaiveDate,
    ) -> Result<Vec<(String, Decimal)>, InventoryError> {
        // Sort: perishable batches by expiry_date first, then non-perishable by mfg_date
        available_batches.sort_by(|a, b| {
            match (a.expiry_date, b.expiry_date) {
                (Some(exp_a), Some(exp_b)) => exp_a.cmp(&exp_b).then_with(|| a.mfg_date.cmp(&b.mfg_date)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.mfg_date.cmp(&b.mfg_date),
            }
        });

        let mut allocations = Vec::new();
        let mut remaining_needed = required_qty;

        for batch in available_batches.iter_mut() {
            if remaining_needed <= Decimal::ZERO {
                break;
            }

            if batch.remaining_qty <= Decimal::ZERO {
                continue;
            }

            // Expiry check
            batch.validate_usable(as_of)?;

            let take_qty = batch.remaining_qty.min(remaining_needed);
            batch.remaining_qty -= take_qty;
            remaining_needed -= take_qty;

            allocations.push((batch.batch_id.clone(), take_qty));
        }

        if remaining_needed > Decimal::ZERO {
            return Err(InventoryError::InsufficientStock {
                requested: required_qty,
                available: required_qty - remaining_needed,
            });
        }

        Ok(allocations)
    }
}
