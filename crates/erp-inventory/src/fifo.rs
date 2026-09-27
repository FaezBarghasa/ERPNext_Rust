use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Inventory domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InventoryError {
    /// Insufficient available stock in FIFO queue.
    #[error("Insufficient stock: requested {requested}, available {available}")]
    InsufficientStock {
        requested: Decimal,
        available: Decimal,
    },
    /// Serial number location mismatch.
    #[error("Serial number '{serial_no}' not found in warehouse '{expected_warehouse}', located in '{actual_warehouse}'")]
    SerialNotInWarehouse {
        serial_no: String,
        expected_warehouse: String,
        actual_warehouse: String,
    },
    /// Expired material batch.
    #[error("Batch '{batch_id}' has expired on {expiry_date}")]
    BatchExpired {
        batch_id: String,
        expiry_date: NaiveDate,
    },
    /// Quantity must be positive.
    #[error("Quantity must be positive, got {0}")]
    InvalidQuantity(Decimal),
    /// Item not found.
    #[error("Item not found: {0}")]
    ItemNotFound(String),
}

/// An individual FIFO inventory batch layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FifoBatchItem {
    /// Available batch quantity.
    pub qty: Decimal,
    /// Unit valuation rate in base currency.
    pub rate: Decimal,
}

/// Immutable Stock Ledger Entry (SLE) recording physical inventory movement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockLedgerEntry {
    /// Primary key identifier.
    pub name: String,
    /// Item code.
    pub item_code: String,
    /// Warehouse identifier.
    pub warehouse: String,
    /// Change in physical quantity (positive for incoming receipt, negative for dispatch).
    pub actual_qty: Decimal,
    /// Incoming valuation rate.
    pub incoming_rate: Decimal,
    /// Cumulative valuation rate after this transaction.
    pub valuation_rate: Decimal,
    /// Net change in total monetary stock value.
    pub stock_value_difference: Decimal,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Originating document type.
    pub voucher_type: String,
    /// Originating document number.
    pub voucher_no: String,
}

/// Consumes quantities from a FIFO batch queue sequentially, calculating exact COGS.
pub fn consume_fifo(
    queue: &mut Vec<FifoBatchItem>,
    mut qty_to_remove: Decimal,
) -> Result<Decimal, InventoryError> {
    if qty_to_remove <= Decimal::ZERO {
        return Err(InventoryError::InvalidQuantity(qty_to_remove));
    }

    let available: Decimal = queue.iter().map(|b| b.qty).sum();
    if available < qty_to_remove {
        return Err(InventoryError::InsufficientStock {
            requested: qty_to_remove,
            available,
        });
    }

    let mut total_cost = Decimal::ZERO;

    while qty_to_remove > Decimal::ZERO {
        let oldest = queue.first_mut().ok_or(InventoryError::InsufficientStock {
            requested: qty_to_remove,
            available: Decimal::ZERO,
        })?;

        if oldest.qty <= qty_to_remove {
            qty_to_remove -= oldest.qty;
            total_cost += oldest.qty * oldest.rate;
            queue.remove(0);
        } else {
            oldest.qty -= qty_to_remove;
            total_cost += qty_to_remove * oldest.rate;
            qty_to_remove = Decimal::ZERO;
        }
    }

    Ok(total_cost)
}

/// Adds incoming receipt batch layer to FIFO queue.
pub fn add_fifo_layer(queue: &mut Vec<FifoBatchItem>, qty: Decimal, rate: Decimal) {
    if qty > Decimal::ZERO {
        queue.push(FifoBatchItem { qty, rate });
    }
}
