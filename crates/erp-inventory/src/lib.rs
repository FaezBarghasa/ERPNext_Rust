pub mod ageing;
pub mod batches;
pub mod catch_weight;
pub mod fifo;
pub mod reconciliation;
pub mod reposting;
pub mod reservation;
pub mod warehouse;

pub use ageing::{AgeingBucket, StockAgeingEngine, StockAgeingSummary, StockReceiptLayer};
pub use batches::{Batch, SerialNo, SerialStatus};
pub use catch_weight::CatchWeightItem;
pub use fifo::{FifoBatchItem, InventoryError, StockLedgerEntry, add_fifo_layer, consume_fifo};
pub use reconciliation::{BatchAuditReport, BatchDiscrepancy, BatchRecalculator};
pub use reposting::{ItemPartitionQueue, ParallelRepostingSettings, RepostItemJob};
pub use reservation::{
    ReservationStatus, ReservationType, StockReservationEngine, StockReservationEntry,
};
pub use warehouse::{
    Warehouse, create_delivery_note_gl_entries, create_purchase_receipt_gl_entries,
    gl_entries_to_journal_entry,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fifo_multi_batch_consumption() {
        let mut queue = vec![
            FifoBatchItem {
                qty: dec!(10.0),
                rate: dec!(10.0),
            }, // Cost = 100
            FifoBatchItem {
                qty: dec!(10.0),
                rate: dec!(20.0),
            }, // Cost = 200
        ];

        // Dispatch 15 units
        let cogs = consume_fifo(&mut queue, dec!(15.0)).expect("FIFO consume failed");

        // 10 units @ 10 = 100, 5 units @ 20 = 100 -> Total = 200
        assert_eq!(cogs, dec!(200.0));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].qty, dec!(5.0));
        assert_eq!(queue[0].rate, dec!(20.0));

        // Attempt consuming more than remaining (5 left, request 6)
        let err = consume_fifo(&mut queue, dec!(6.0));
        assert!(matches!(err, Err(InventoryError::InsufficientStock { .. })));
    }

    #[test]
    fn test_warehouse_gl_sync_and_journal_balance() {
        let wh = Warehouse {
            name: "Main Stores - ACME".into(),
            stock_in_hand_account: "1300 - Stock In Hand".into(),
            stock_received_but_not_billed_account: "2100 - Stock Received But Not Billed".into(),
            default_cogs_account: "5100 - Cost of Goods Sold".into(),
            company: "Acme Corp".into(),
        };

        let date = NaiveDate::from_ymd_opt(2026, 9, 27).unwrap();

        // 1. Purchase Receipt GL Entries
        let pr_entries = create_purchase_receipt_gl_entries(&wh, dec!(500.0), date, "PR-001");
        let pr_jv = gl_entries_to_journal_entry(&pr_entries, date, &wh.company, "Receipt PR-001");
        assert!(pr_jv.validate_balance().is_ok());

        // 2. Delivery Note GL Entries
        let dn_entries = create_delivery_note_gl_entries(&wh, dec!(200.0), date, "DN-001");
        let dn_jv = gl_entries_to_journal_entry(&dn_entries, date, &wh.company, "Delivery DN-001");
        assert!(dn_jv.validate_balance().is_ok());
    }

    #[test]
    fn test_serial_and_batch_validation() {
        let serial = SerialNo {
            serial_no: "SN-98765".into(),
            item_code: "LAPTOP-01".into(),
            warehouse: "Warehouse A".into(),
            status: SerialStatus::Active,
        };

        assert!(serial.validate_dispatch_from("Warehouse A").is_ok());
        assert!(matches!(
            serial.validate_dispatch_from("Warehouse B"),
            Err(InventoryError::SerialNotInWarehouse { .. })
        ));

        let batch = Batch {
            batch_id: "BATCH-2026-01".into(),
            item_code: "CHEMICAL-X".into(),
            mfg_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            expiry_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            remaining_qty: dec!(50.0),
        };

        let valid_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        assert!(batch.validate_usable(valid_date).is_ok());

        let expired_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        assert!(matches!(
            batch.validate_usable(expired_date),
            Err(InventoryError::BatchExpired { .. })
        ));
    }
}
