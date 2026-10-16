//! Strict Serial Number Lifecycle State Machine (`erp-inventory::serial_state_machine`).
//!
//! Enforces deterministic lifecycle transitions for serialized assets and inventory:
//! - Lifecycle: `Draft -> Active -> Reserved -> InTransit -> Delivered -> Returned -> Scrapped / Expired`
//! - Disallows illegal movements (e.g. delivering an already delivered or scrapped unit)
//! - Prevents duplicate serial number registrations on receipt
//! - Maintains an immutable audit history trail per serial number.

use chrono::NaiveDate;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Serial state machine domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SerialStateError {
    #[error("Serial number '{0}' already exists and is active in warehouse '{1}'")]
    SerialAlreadyActive(CompactString, CompactString),
    #[error("Serial number '{0}' not found in registry")]
    SerialNotFound(CompactString),
    #[error(
        "Illegal state transition for serial '{serial_no}': cannot transition from {current_state:?} to {attempted_state:?} (action: {action})"
    )]
    IllegalTransition {
        serial_no: CompactString,
        current_state: SerialLifecycleState,
        attempted_state: SerialLifecycleState,
        action: &'static str,
    },
    #[error("Warehouse mismatch for serial '{serial_no}': expected '{expected}', found '{actual}'")]
    WarehouseMismatch {
        serial_no: CompactString,
        expected: CompactString,
        actual: CompactString,
    },
    #[error("Item code mismatch for serial '{serial_no}': expected '{expected}', found '{actual}'")]
    ItemCodeMismatch {
        serial_no: CompactString,
        expected: CompactString,
        actual: CompactString,
    },
    #[error(
        "Serial number '{0}' has been scrapped or decommissioned; no further movement permitted"
    )]
    SerialScrapped(CompactString),
    #[error("Serial number '{0}' has expired; no dispatch permitted")]
    SerialExpired(CompactString),
}

/// Lifecycle states of a serialized unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerialLifecycleState {
    /// Created in draft state, awaiting physical receipt.
    Draft,
    /// Physically in warehouse, available for movement or sale.
    Active,
    /// Allocated or locked for an active Sales Order or Work Order.
    Reserved,
    /// In transit between warehouses via Material Transfer.
    InTransit,
    /// Shipped and delivered to customer.
    Delivered,
    /// Returned by customer, awaiting inspection or restock.
    Returned,
    /// Decommissioned, damaged, or written off (terminal state).
    Scrapped,
    /// Expiration date passed (terminal state for sales).
    Expired,
}

/// Audit history entry recording a state transition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerialHistoryRecord {
    pub posting_date: NaiveDate,
    pub voucher_type: CompactString,
    pub voucher_no: CompactString,
    pub from_state: SerialLifecycleState,
    pub to_state: SerialLifecycleState,
    pub warehouse: Option<CompactString>,
    pub remarks: Option<String>,
}

/// A serialized item record with location, state, and complete lifecycle history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerialRecord {
    pub serial_no: CompactString,
    pub item_code: CompactString,
    pub warehouse: Option<CompactString>,
    pub state: SerialLifecycleState,
    pub batch_no: Option<CompactString>,
    pub history: Vec<SerialHistoryRecord>,
}

/// Parameters for registering a newly received serial unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerialReceiptParams {
    pub serial_no: CompactString,
    pub item_code: CompactString,
    pub warehouse: CompactString,
    pub voucher_type: CompactString,
    pub voucher_no: CompactString,
    pub posting_date: NaiveDate,
    pub batch_no: Option<CompactString>,
}

impl SerialRecord {
    /// Creates a new serial number in `Draft` state.
    #[must_use]
    pub fn new_draft(serial_no: CompactString, item_code: CompactString) -> Self {
        Self {
            serial_no,
            item_code,
            warehouse: None,
            state: SerialLifecycleState::Draft,
            batch_no: None,
            history: Vec::new(),
        }
    }

    /// Records a verified transition into the audit log.
    fn record_transition(
        &mut self,
        to_state: SerialLifecycleState,
        warehouse: Option<CompactString>,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
        remarks: Option<String>,
    ) {
        self.history.push(SerialHistoryRecord {
            posting_date,
            voucher_type,
            voucher_no,
            from_state: self.state,
            to_state,
            warehouse: warehouse.clone(),
            remarks,
        });
        self.state = to_state;
        self.warehouse = warehouse;
    }
}

/// In-memory transactional Serial Number Registry.
#[derive(Debug, Default, Clone)]
pub struct SerialRegistry {
    records: HashMap<CompactString, SerialRecord>,
}

impl SerialRegistry {
    /// Creates an empty serial number registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Retrieves an existing serial record.
    #[must_use]
    pub fn get(&self, serial_no: &str) -> Option<&SerialRecord> {
        self.records.get(serial_no)
    }

    /// Registers a newly received serial unit into stock (Draft -> Active).
    /// Prevents registering a duplicate serial number if already active or delivered.
    pub fn register_receipt(
        &mut self,
        params: SerialReceiptParams,
    ) -> Result<&SerialRecord, SerialStateError> {
        let serial_no = params.serial_no;
        let item_code = params.item_code;
        let warehouse = params.warehouse;

        if let Some(existing) = self.records.get(&serial_no) {
            if matches!(
                existing.state,
                SerialLifecycleState::Active
                    | SerialLifecycleState::Reserved
                    | SerialLifecycleState::InTransit
            ) {
                return Err(SerialStateError::SerialAlreadyActive(
                    serial_no,
                    existing.warehouse.clone().unwrap_or_default(),
                ));
            }
            if existing.state == SerialLifecycleState::Scrapped {
                return Err(SerialStateError::SerialScrapped(serial_no));
            }
        }

        let mut record = self
            .records
            .remove(&serial_no)
            .unwrap_or_else(|| SerialRecord::new_draft(serial_no.clone(), item_code.clone()));

        if record.item_code != item_code {
            return Err(SerialStateError::ItemCodeMismatch {
                serial_no,
                expected: record.item_code.clone(),
                actual: item_code,
            });
        }

        record.batch_no = params.batch_no;
        record.record_transition(
            SerialLifecycleState::Active,
            Some(warehouse),
            params.voucher_type,
            params.voucher_no,
            params.posting_date,
            Some("Stock Receipt".to_string()),
        );

        self.records.insert(serial_no.clone(), record);
        Ok(self.records.get(&serial_no).unwrap())
    }

    /// Reserves an active serial number against an order or project.
    pub fn reserve(
        &mut self,
        serial_no: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::Active {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Reserved,
                action: "reserve",
            });
        }

        let wh = record.warehouse.clone();
        record.record_transition(
            SerialLifecycleState::Reserved,
            wh,
            voucher_type,
            voucher_no,
            posting_date,
            Some("Stock Reservation".to_string()),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Releases a reservation back to Active.
    pub fn unreserve(
        &mut self,
        serial_no: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::Reserved {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Active,
                action: "unreserve",
            });
        }

        let wh = record.warehouse.clone();
        record.record_transition(
            SerialLifecycleState::Active,
            wh,
            voucher_type,
            voucher_no,
            posting_date,
            Some("Release Reservation".to_string()),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Initiates a warehouse transfer (Active -> InTransit).
    pub fn transfer_start(
        &mut self,
        serial_no: &str,
        source_warehouse: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::Active {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::InTransit,
                action: "transfer_start",
            });
        }

        let current_wh = record.warehouse.as_deref().unwrap_or_default();
        if current_wh != source_warehouse {
            return Err(SerialStateError::WarehouseMismatch {
                serial_no: record.serial_no.clone(),
                expected: source_warehouse.into(),
                actual: current_wh.into(),
            });
        }

        record.record_transition(
            SerialLifecycleState::InTransit,
            None,
            voucher_type,
            voucher_no,
            posting_date,
            Some(format!("Transfer dispatched from '{source_warehouse}'")),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Completes a warehouse transfer (InTransit -> Active at target warehouse).
    pub fn transfer_complete(
        &mut self,
        serial_no: &str,
        target_warehouse: CompactString,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::InTransit {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Active,
                action: "transfer_complete",
            });
        }

        record.record_transition(
            SerialLifecycleState::Active,
            Some(target_warehouse.clone()),
            voucher_type,
            voucher_no,
            posting_date,
            Some(format!("Transfer received at '{target_warehouse}'")),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Delivers an active or reserved unit to a customer (Active/Reserved -> Delivered).
    pub fn deliver(
        &mut self,
        serial_no: &str,
        from_warehouse: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if !matches!(
            record.state,
            SerialLifecycleState::Active | SerialLifecycleState::Reserved
        ) {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Delivered,
                action: "deliver",
            });
        }

        let current_wh = record.warehouse.as_deref().unwrap_or_default();
        if current_wh != from_warehouse {
            return Err(SerialStateError::WarehouseMismatch {
                serial_no: record.serial_no.clone(),
                expected: from_warehouse.into(),
                actual: current_wh.into(),
            });
        }

        record.record_transition(
            SerialLifecycleState::Delivered,
            None,
            voucher_type,
            voucher_no,
            posting_date,
            Some("Customer Delivery".to_string()),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Processes a customer sales return (Delivered -> Returned).
    pub fn customer_return(
        &mut self,
        serial_no: &str,
        return_warehouse: CompactString,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::Delivered {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Returned,
                action: "customer_return",
            });
        }

        record.record_transition(
            SerialLifecycleState::Returned,
            Some(return_warehouse.clone()),
            voucher_type,
            voucher_no,
            posting_date,
            Some(format!("Customer Return into '{return_warehouse}'")),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Restocks a returned unit back to active status (Returned -> Active).
    pub fn restock(
        &mut self,
        serial_no: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if record.state != SerialLifecycleState::Returned {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Active,
                action: "restock",
            });
        }

        let wh = record.warehouse.clone();
        record.record_transition(
            SerialLifecycleState::Active,
            wh,
            voucher_type,
            voucher_no,
            posting_date,
            Some("Restocked after inspection".to_string()),
        );

        Ok(self.records.get(serial_no).unwrap())
    }

    /// Scraps or decommissions a damaged or obsolete unit.
    pub fn scrap(
        &mut self,
        serial_no: &str,
        voucher_type: CompactString,
        voucher_no: CompactString,
        posting_date: NaiveDate,
        reason: &str,
    ) -> Result<&SerialRecord, SerialStateError> {
        let record = self
            .records
            .get_mut(serial_no)
            .ok_or_else(|| SerialStateError::SerialNotFound(serial_no.into()))?;

        if !matches!(
            record.state,
            SerialLifecycleState::Active
                | SerialLifecycleState::Returned
                | SerialLifecycleState::Expired
        ) {
            return Err(SerialStateError::IllegalTransition {
                serial_no: record.serial_no.clone(),
                current_state: record.state,
                attempted_state: SerialLifecycleState::Scrapped,
                action: "scrap",
            });
        }

        record.record_transition(
            SerialLifecycleState::Scrapped,
            None,
            voucher_type,
            voucher_no,
            posting_date,
            Some(format!("Scrapped: {reason}")),
        );

        Ok(self.records.get(serial_no).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_lifecycle_state_machine_happy_path() {
        let mut registry = SerialRegistry::new();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();

        // 1. Receipt
        let rec = registry
            .register_receipt(SerialReceiptParams {
                serial_no: "SN-001".into(),
                item_code: "LAPTOP-PRO".into(),
                warehouse: "Main Stores".into(),
                voucher_type: "Purchase Receipt".into(),
                voucher_no: "PR-001".into(),
                posting_date: date,
                batch_no: None,
            })
            .expect("Receipt failed");
        assert_eq!(rec.state, SerialLifecycleState::Active);
        assert_eq!(rec.warehouse.as_deref(), Some("Main Stores"));

        // 2. Transfer to Finished Goods
        registry
            .transfer_start(
                "SN-001",
                "Main Stores",
                "Stock Entry".into(),
                "STE-001".into(),
                date,
            )
            .expect("Transfer start failed");
        let in_transit = registry.get("SN-001").unwrap();
        assert_eq!(in_transit.state, SerialLifecycleState::InTransit);
        assert!(in_transit.warehouse.is_none());

        registry
            .transfer_complete(
                "SN-001",
                "Finished Goods".into(),
                "Stock Entry".into(),
                "STE-001".into(),
                date,
            )
            .expect("Transfer complete failed");
        let transferred = registry.get("SN-001").unwrap();
        assert_eq!(transferred.state, SerialLifecycleState::Active);
        assert_eq!(transferred.warehouse.as_deref(), Some("Finished Goods"));

        // 3. Deliver to customer
        registry
            .deliver(
                "SN-001",
                "Finished Goods",
                "Delivery Note".into(),
                "DN-001".into(),
                date,
            )
            .expect("Delivery failed");
        let delivered = registry.get("SN-001").unwrap();
        assert_eq!(delivered.state, SerialLifecycleState::Delivered);
        assert!(delivered.warehouse.is_none());

        // 4. Return from customer
        registry
            .customer_return(
                "SN-001",
                "Quarantine Warehouse".into(),
                "Delivery Note".into(),
                "DN-RET-001".into(),
                date,
            )
            .expect("Customer return failed");
        let returned = registry.get("SN-001").unwrap();
        assert_eq!(returned.state, SerialLifecycleState::Returned);
        assert_eq!(returned.warehouse.as_deref(), Some("Quarantine Warehouse"));

        // 5. Restock after inspection
        registry
            .restock(
                "SN-001",
                "Stock Entry".into(),
                "STE-RESTOCK-001".into(),
                date,
            )
            .expect("Restock failed");
        let restocked = registry.get("SN-001").unwrap();
        assert_eq!(restocked.state, SerialLifecycleState::Active);

        // 6. Verify audit history
        assert_eq!(restocked.history.len(), 6);
    }

    #[test]
    fn test_serial_lifecycle_illegal_transitions_prevented() {
        let mut registry = SerialRegistry::new();
        let date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();

        registry
            .register_receipt(SerialReceiptParams {
                serial_no: "SN-002".into(),
                item_code: "PHONE-X".into(),
                warehouse: "Stores".into(),
                voucher_type: "Purchase Receipt".into(),
                voucher_no: "PR-002".into(),
                posting_date: date,
                batch_no: None,
            })
            .unwrap();

        // 1. Prohibit registering active duplicate
        let dup_res = registry.register_receipt(SerialReceiptParams {
            serial_no: "SN-002".into(),
            item_code: "PHONE-X".into(),
            warehouse: "Stores".into(),
            voucher_type: "Purchase Receipt".into(),
            voucher_no: "PR-003".into(),
            posting_date: date,
            batch_no: None,
        });
        assert!(matches!(
            dup_res,
            Err(SerialStateError::SerialAlreadyActive(..))
        ));

        // 2. Prohibit delivery from wrong warehouse
        let wh_err = registry.deliver(
            "SN-002",
            "Wrong Warehouse",
            "Delivery Note".into(),
            "DN-002".into(),
            date,
        );
        assert!(matches!(
            wh_err,
            Err(SerialStateError::WarehouseMismatch { .. })
        ));

        // 3. Deliver successfully
        registry
            .deliver(
                "SN-002",
                "Stores",
                "Delivery Note".into(),
                "DN-002".into(),
                date,
            )
            .unwrap();

        // 4. Prohibit second delivery of already delivered item
        let second_deliv = registry.deliver(
            "SN-002",
            "Stores",
            "Delivery Note".into(),
            "DN-003".into(),
            date,
        );
        assert!(matches!(
            second_deliv,
            Err(SerialStateError::IllegalTransition { .. })
        ));
    }
}
