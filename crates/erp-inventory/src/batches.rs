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
}
