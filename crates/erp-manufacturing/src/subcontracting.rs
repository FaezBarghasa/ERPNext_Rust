//! Transactional Subcontracting Order & Subcontracting Receipt Controller (`erp-manufacturing::subcontracting`).
//!
//! Implements canonical upstream ERPNext Subcontracting business logic:
//! - Subcontracting Order specifying finished items to produce and raw materials supplied to vendor
//! - Subcontracting Receipt accepting finished goods into inventory
//! - Reconciles consumed raw materials from the Subcontractor / Supplier Warehouse
//! - Computes finished goods unit incoming rate = vendor service charges + consumed raw material cost
//! - Generates dual-ended Stock Ledger Entries (SLEs) for raw material consumption and finished goods receipt
//! - Generates balanced General Ledger (GL) double-entry transactions (Debit FG Stock In Hand, Credit Raw Materials Stock In Hand, Credit Subcontracting Accrual / Stock Received But Not Billed).

use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::GlEntry;
use erp_inventory::StockLedgerEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for Subcontracting operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SubcontractingError {
    #[error("Subcontracting voucher must contain at least one line item")]
    EmptyItems,
    #[error("Cannot submit document with status {0}, expected Draft (0)")]
    InvalidDocStatus(i32),
    #[error("Non-positive quantity for item '{item_code}': {qty}")]
    InvalidQuantity {
        item_code: CompactString,
        qty: Decimal,
    },
    #[error("No raw materials supplied or consumed for finished item '{0}'")]
    MissingSuppliedItems(CompactString),
    #[error("Warehouse GL account not found for warehouse '{0}'")]
    WarehouseAccountNotFound(CompactString),
    #[error("Unbalanced GL double-entries: total debit {total_debit} != total credit {total_credit}")]
    UnbalancedGlEntries {
        total_debit: Decimal,
        total_credit: Decimal,
    },
}

/// A finished good line item on a Subcontracting Order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingOrderItem {
    pub item_code: CompactString,
    pub qty: Decimal,
    pub rate: Decimal, // Service processing fee per unit
    pub amount: Decimal,
    pub bom_no: Option<CompactString>,
}

/// A raw material component supplied to the vendor on a Subcontracting Order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingSuppliedItem {
    pub main_item_code: CompactString,
    pub rm_item_code: CompactString,
    pub required_qty: Decimal,
    pub consumed_qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
}

/// Canonical ERPNext Subcontracting Order document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingOrder {
    pub name: CompactString,
    pub supplier: CompactString,
    pub posting_date: NaiveDate,
    pub company: CompactString,
    pub items: Vec<SubcontractingOrderItem>,
    pub supplied_items: Vec<SubcontractingSuppliedItem>,
    pub total_service_cost: Decimal,
    pub docstatus: i32,
    pub remarks: Option<String>,
}

impl SubcontractingOrder {
    /// Validates quantities and computes total service costs.
    pub fn validate(&mut self) -> Result<(), SubcontractingError> {
        if self.items.is_empty() {
            return Err(SubcontractingError::EmptyItems);
        }

        let mut total_cost = Decimal::ZERO;
        for item in &mut self.items {
            if item.qty <= Decimal::ZERO {
                return Err(SubcontractingError::InvalidQuantity {
                    item_code: item.item_code.clone(),
                    qty: item.qty,
                });
            }
            item.amount = item.qty * item.rate;
            total_cost += item.amount;
        }

        for rm in &mut self.supplied_items {
            rm.amount = rm.required_qty * rm.rate;
        }

        self.total_service_cost = total_cost;
        Ok(())
    }

    /// Submits the Subcontracting Order.
    pub fn submit(&mut self) -> Result<(), SubcontractingError> {
        if self.docstatus != 0 {
            return Err(SubcontractingError::InvalidDocStatus(self.docstatus));
        }
        self.validate()?;
        self.docstatus = 1;
        Ok(())
    }
}

/// An individual finished good line item received in Subcontracting Receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingReceiptItem {
    pub item_code: CompactString,
    pub qty: Decimal,
    pub service_rate: Decimal,
    pub service_amount: Decimal,
    pub rm_cost_amount: Decimal,
    pub incoming_rate: Decimal, // (service_amount + rm_cost_amount) / qty
    pub amount: Decimal,        // service_amount + rm_cost_amount
}

/// An individual raw material component consumed during subcontracting execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingReceiptSuppliedItem {
    pub main_item_code: CompactString,
    pub rm_item_code: CompactString,
    pub consumed_qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
}

/// Canonical ERPNext Subcontracting Receipt document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingReceipt {
    pub name: CompactString,
    pub subcontracting_order: CompactString,
    pub supplier: CompactString,
    pub supplier_warehouse: CompactString, // Where raw materials reside and are consumed
    pub accepted_warehouse: CompactString, // Where finished goods are placed
    pub posting_date: NaiveDate,
    pub company: CompactString,
    pub items: Vec<SubcontractingReceiptItem>,
    pub supplied_items: Vec<SubcontractingReceiptSuppliedItem>,
    pub total_service_cost: Decimal,
    pub total_rm_cost: Decimal,
    pub grand_total: Decimal,
    pub docstatus: i32,
    pub remarks: Option<String>,
}

impl SubcontractingReceipt {
    /// Validates lines, aggregates consumed raw materials onto finished goods, and calculates incoming rate.
    pub fn validate(&mut self) -> Result<(), SubcontractingError> {
        if self.items.is_empty() {
            return Err(SubcontractingError::EmptyItems);
        }

        let mut total_rm_cost = Decimal::ZERO;
        for rm in &mut self.supplied_items {
            if rm.consumed_qty < Decimal::ZERO {
                return Err(SubcontractingError::InvalidQuantity {
                    item_code: rm.rm_item_code.clone(),
                    qty: rm.consumed_qty,
                });
            }
            rm.amount = rm.consumed_qty * rm.rate;
            total_rm_cost += rm.amount;
        }

        let mut total_service = Decimal::ZERO;
        let mut total_grand = Decimal::ZERO;

        for item in &mut self.items {
            if item.qty <= Decimal::ZERO {
                return Err(SubcontractingError::InvalidQuantity {
                    item_code: item.item_code.clone(),
                    qty: item.qty,
                });
            }

            item.service_amount = item.qty * item.service_rate;

            // Compute raw material cost allocated to this finished good
            let item_rm_cost: Decimal = self
                .supplied_items
                .iter()
                .filter(|rm| rm.main_item_code == item.item_code)
                .map(|rm| rm.amount)
                .sum();

            item.rm_cost_amount = item_rm_cost;
            item.amount = item.service_amount + item.rm_cost_amount;
            item.incoming_rate = item.amount / item.qty;

            total_service += item.service_amount;
            total_grand += item.amount;
        }

        self.total_service_cost = total_service;
        self.total_rm_cost = total_rm_cost;
        self.grand_total = total_grand;

        Ok(())
    }

    /// Submits the Subcontracting Receipt.
    pub fn submit(&mut self) -> Result<(), SubcontractingError> {
        if self.docstatus != 0 {
            return Err(SubcontractingError::InvalidDocStatus(self.docstatus));
        }
        self.validate()?;
        self.docstatus = 1;
        Ok(())
    }

    /// Generates Stock Ledger Entries (SLEs):
    /// - Outward from Supplier Warehouse for each consumed raw material.
    /// - Inward into Accepted Warehouse for each received finished good.
    pub fn generate_stock_ledger_entries(&self) -> Vec<StockLedgerEntry> {
        let mut sles = Vec::new();
        let voucher_type = "Subcontracting Receipt".to_string();
        let voucher_no = self.name.to_string();

        // 1. Outward SLEs for consumed raw materials
        for (idx, rm) in self.supplied_items.iter().enumerate() {
            if rm.consumed_qty > Decimal::ZERO {
                sles.push(StockLedgerEntry {
                    name: format!("{}-rm-{}", self.name, idx + 1),
                    item_code: rm.rm_item_code.to_string(),
                    warehouse: self.supplier_warehouse.to_string(),
                    actual_qty: -rm.consumed_qty,
                    incoming_rate: rm.rate,
                    valuation_rate: rm.rate,
                    stock_value_difference: -rm.amount,
                    posting_date: self.posting_date,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                });
            }
        }

        // 2. Inward SLEs for received finished items
        for (idx, item) in self.items.iter().enumerate() {
            sles.push(StockLedgerEntry {
                name: format!("{}-fg-{}", self.name, idx + 1),
                item_code: item.item_code.to_string(),
                warehouse: self.accepted_warehouse.to_string(),
                actual_qty: item.qty,
                incoming_rate: item.incoming_rate,
                valuation_rate: item.incoming_rate,
                stock_value_difference: item.amount,
                posting_date: self.posting_date,
                voucher_type: voucher_type.clone(),
                voucher_no: voucher_no.clone(),
            });
        }

        sles
    }

    /// Generates balanced double-entry GL records:
    /// - Debit: Accepted Warehouse Stock In Hand (Grand Total = RM + Service)
    /// - Credit: Supplier Warehouse Stock In Hand (Consumed Raw Materials)
    /// - Credit: Subcontracting Service Accrual / Stock Received But Not Billed (Vendor Service Charge)
    pub fn generate_gl_entries<F>(
        &self,
        service_accrual_account: &str,
        mut get_warehouse_account: F,
    ) -> Result<Vec<GlEntry>, SubcontractingError>
    where
        F: FnMut(&str) -> Result<CompactString, SubcontractingError>,
    {
        let mut entries = Vec::new();
        let voucher_type = "Subcontracting Receipt".to_string();
        let voucher_no = self.name.to_string();
        let company = self.company.to_string();

        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        // 1. Debit Finished Goods Stock In Hand
        let fg_account = get_warehouse_account(self.accepted_warehouse.as_str())?;
        entries.push(GlEntry {
            name: format!("{}-fg", self.name),
            posting_date: self.posting_date,
            account: fg_account.to_string(),
            debit: self.grand_total,
            credit: Decimal::ZERO,
            voucher_type: voucher_type.clone(),
            voucher_no: voucher_no.clone(),
            party_type: None,
            party: None,
            company: company.clone(),
        });
        total_debit += self.grand_total;

        // 2. Credit Supplier Warehouse Stock In Hand for consumed raw materials
        if self.total_rm_cost > Decimal::ZERO {
            let rm_account = get_warehouse_account(self.supplier_warehouse.as_str())?;
            entries.push(GlEntry {
                name: format!("{}-rm", self.name),
                posting_date: self.posting_date,
                account: rm_account.to_string(),
                debit: Decimal::ZERO,
                credit: self.total_rm_cost,
                voucher_type: voucher_type.clone(),
                voucher_no: voucher_no.clone(),
                party_type: None,
                party: None,
                company: company.clone(),
            });
            total_credit += self.total_rm_cost;
        }

        // 3. Credit Service Charges / Stock Received But Not Billed
        if self.total_service_cost > Decimal::ZERO {
            entries.push(GlEntry {
                name: format!("{}-service", self.name),
                posting_date: self.posting_date,
                account: service_accrual_account.to_string(),
                debit: Decimal::ZERO,
                credit: self.total_service_cost,
                voucher_type: voucher_type.clone(),
                voucher_no: voucher_no.clone(),
                party_type: Some("Supplier".into()),
                party: Some(self.supplier.to_string()),
                company: company.clone(),
            });
            total_credit += self.total_service_cost;
        }

        if total_debit != total_credit {
            return Err(SubcontractingError::UnbalancedGlEntries {
                total_debit,
                total_credit,
            });
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_subcontracting_lifecycle_and_cost_aggregation() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        // 1. Subcontracting Order: 100 Finished Motor Assemblies
        // Service rate = $15.00/unit ($1,500 total service fee)
        // Raw materials:
        // - 100 Stator Cores @ $10.00 = $1,000
        // - 100 Copper Windings @ $5.00 = $500
        let mut sco = SubcontractingOrder {
            name: "SCO-2026-0001".into(),
            supplier: "Apex Precision Motors".into(),
            posting_date: date,
            company: "Robotics Corp".into(),
            items: vec![SubcontractingOrderItem {
                item_code: "MOTOR-ASSY".into(),
                qty: dec!(100.0),
                rate: dec!(15.00),
                amount: Decimal::ZERO,
                bom_no: Some("BOM-MOTOR-01".into()),
            }],
            supplied_items: vec![
                SubcontractingSuppliedItem {
                    main_item_code: "MOTOR-ASSY".into(),
                    rm_item_code: "STATOR-CORE".into(),
                    required_qty: dec!(100.0),
                    consumed_qty: Decimal::ZERO,
                    rate: dec!(10.00),
                    amount: Decimal::ZERO,
                },
                SubcontractingSuppliedItem {
                    main_item_code: "MOTOR-ASSY".into(),
                    rm_item_code: "COPPER-WINDING".into(),
                    required_qty: dec!(100.0),
                    consumed_qty: Decimal::ZERO,
                    rate: dec!(5.00),
                    amount: Decimal::ZERO,
                },
            ],
            total_service_cost: Decimal::ZERO,
            docstatus: 0,
            remarks: Some("Motor winding subcontracting contract".into()),
        };

        sco.submit().expect("Subcontracting order submission failed");
        assert_eq!(sco.docstatus, 1);
        assert_eq!(sco.total_service_cost, dec!(1500.00));

        // 2. Subcontracting Receipt: Receiving 100 finished motors
        let mut scr = SubcontractingReceipt {
            name: "SCR-2026-0001".into(),
            subcontracting_order: "SCO-2026-0001".into(),
            supplier: "Apex Precision Motors".into(),
            supplier_warehouse: "Apex Warehouse - ACME".into(),
            accepted_warehouse: "Finished Goods - ACME".into(),
            posting_date: date,
            company: "Robotics Corp".into(),
            items: vec![SubcontractingReceiptItem {
                item_code: "MOTOR-ASSY".into(),
                qty: dec!(100.0),
                service_rate: dec!(15.00),
                service_amount: Decimal::ZERO,
                rm_cost_amount: Decimal::ZERO,
                incoming_rate: Decimal::ZERO,
                amount: Decimal::ZERO,
            }],
            supplied_items: vec![
                SubcontractingReceiptSuppliedItem {
                    main_item_code: "MOTOR-ASSY".into(),
                    rm_item_code: "STATOR-CORE".into(),
                    consumed_qty: dec!(100.0),
                    rate: dec!(10.00),
                    amount: Decimal::ZERO,
                },
                SubcontractingReceiptSuppliedItem {
                    main_item_code: "MOTOR-ASSY".into(),
                    rm_item_code: "COPPER-WINDING".into(),
                    consumed_qty: dec!(100.0),
                    rate: dec!(5.00),
                    amount: Decimal::ZERO,
                },
            ],
            total_service_cost: Decimal::ZERO,
            total_rm_cost: Decimal::ZERO,
            grand_total: Decimal::ZERO,
            docstatus: 0,
            remarks: Some("Receipt of finished motor assemblies".into()),
        };

        scr.submit().expect("Subcontracting receipt submit failed");
        assert_eq!(scr.docstatus, 1);
        assert_eq!(scr.total_service_cost, dec!(1500.00));
        assert_eq!(scr.total_rm_cost, dec!(1500.00));
        assert_eq!(scr.grand_total, dec!(3000.00));

        // Finished Good incoming unit rate = ($1,500 service + $1,500 RM) / 100 = $30.00/unit
        assert_eq!(scr.items[0].incoming_rate, dec!(30.00));
        assert_eq!(scr.items[0].amount, dec!(3000.00));

        // Verify SLEs
        let sles = scr.generate_stock_ledger_entries();
        assert_eq!(sles.len(), 3);
        assert_eq!(sles[0].warehouse, "Apex Warehouse - ACME");
        assert_eq!(sles[0].actual_qty, dec!(-100.0));
        assert_eq!(sles[0].stock_value_difference, dec!(-1000.00));

        assert_eq!(sles[1].warehouse, "Apex Warehouse - ACME");
        assert_eq!(sles[1].actual_qty, dec!(-100.0));
        assert_eq!(sles[1].stock_value_difference, dec!(-500.00));

        assert_eq!(sles[2].warehouse, "Finished Goods - ACME");
        assert_eq!(sles[2].actual_qty, dec!(100.0));
        assert_eq!(sles[2].stock_value_difference, dec!(3000.00));

        // Verify balanced GL entries
        let gl_entries = scr
            .generate_gl_entries(
                "2120 - Stock Received But Not Billed",
                |wh| match wh {
                    "Apex Warehouse - ACME" => Ok("1310 - Subcontractor Stock".into()),
                    "Finished Goods - ACME" => Ok("1320 - Finished Goods Inventory".into()),
                    _ => Err(SubcontractingError::WarehouseAccountNotFound(wh.into())),
                },
            )
            .expect("GL generation failed");

        assert_eq!(gl_entries.len(), 3);
        let total_debit: Decimal = gl_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = gl_entries.iter().map(|e| e.credit).sum();
        assert_eq!(total_debit, total_credit);
        assert_eq!(total_debit, dec!(3000.00));
    }
}
