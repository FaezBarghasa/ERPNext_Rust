//! Transactional Stock Entry Controller (`erp-inventory::stock_entry`).
//!
//! Implements canonical upstream ERPNext Stock Entry business logic:
//! - Submittable inventory movement document (Draft -> Submitted -> Cancelled)
//! - Multiple operational purposes: `MaterialReceipt`, `MaterialIssue`, `MaterialTransfer`, `Manufacture`, `Repack`
//! - Validates warehouse requirements, negative stock prohibitions, and quantities
//! - Generates balanced Stock Ledger Entries (SLEs) for dual-ended movements
//! - Generates strictly balanced General Ledger (GL) double entries (Stock In Hand, Stock Adjustment, COGS, Overhead Expenses).

use crate::fifo::StockLedgerEntry;
use chrono::NaiveDate;
use compact_str::CompactString;
use erp_accounting::GlEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for Stock Entry transactions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StockEntryError {
    #[error("Stock Entry must contain at least one line item")]
    EmptyItems,
    #[error("Cannot submit document with status {0}, expected Draft (0)")]
    InvalidDocStatus(i32),
    #[error("Line item '{item_code}' has non-positive quantity: {qty}")]
    InvalidQuantity {
        item_code: CompactString,
        qty: Decimal,
    },
    #[error("Line item '{0}' requires a source warehouse")]
    MissingSourceWarehouse(CompactString),
    #[error("Line item '{0}' requires a target warehouse")]
    MissingTargetWarehouse(CompactString),
    #[error(
        "Source and target warehouse cannot be identical for item '{item_code}': '{warehouse}'"
    )]
    IdenticalSourceAndTargetWarehouse {
        item_code: CompactString,
        warehouse: CompactString,
    },
    #[error("Invalid purpose configuration for {purpose:?}: {reason}")]
    InvalidPurposeConfiguration {
        purpose: StockEntryPurpose,
        reason: String,
    },
    #[error(
        "Insufficient stock for item '{item_code}' at warehouse '{warehouse}': available {available}, required {required}"
    )]
    NegativeStockDetected {
        item_code: CompactString,
        warehouse: CompactString,
        available: Decimal,
        required: Decimal,
    },
    #[error("Warehouse account not found for warehouse '{0}'")]
    WarehouseAccountNotFound(CompactString),
    #[error(
        "Unbalanced GL entries generated: total debit {total_debit} != total credit {total_credit}"
    )]
    UnbalancedGlEntries {
        total_debit: Decimal,
        total_credit: Decimal,
    },
}

/// Operational purpose of the Stock Entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockEntryPurpose {
    /// Inward stock receipt into a target warehouse.
    MaterialReceipt,
    /// Outward stock issue from a source warehouse.
    MaterialIssue,
    /// Stock transfer between distinct source and target warehouses.
    MaterialTransfer,
    /// Production order converting raw materials into finished goods and scrap.
    Manufacture,
    /// Conversion / repack of stock items.
    Repack,
}

/// An individual line item in a Stock Entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockEntryItem {
    pub item_code: CompactString,
    pub item_name: CompactString,
    pub s_warehouse: Option<CompactString>,
    pub t_warehouse: Option<CompactString>,
    pub qty: Decimal,
    pub basic_rate: Decimal,
    pub amount: Decimal,
    pub serial_nos: Vec<CompactString>,
    pub batch_no: Option<CompactString>,
    pub expense_account: Option<CompactString>,
    pub cost_center: Option<CompactString>,
    pub is_finished_item: bool,
    pub is_scrap_item: bool,
}

/// Additional operating cost line (e.g. Electricity, Direct Labor, Machine Depreciation) in Manufacture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockEntryAdditionalCost {
    pub expense_account: CompactString,
    pub description: CompactString,
    pub amount: Decimal,
}

/// Canonical ERPNext Stock Entry document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockEntry {
    pub name: CompactString,
    pub purpose: StockEntryPurpose,
    pub posting_date: NaiveDate,
    pub company: CompactString,
    pub from_warehouse: Option<CompactString>,
    pub to_warehouse: Option<CompactString>,
    pub items: Vec<StockEntryItem>,
    pub additional_costs: Vec<StockEntryAdditionalCost>,
    pub total_incoming_value: Decimal,
    pub total_outgoing_value: Decimal,
    pub value_difference: Decimal,
    pub total_additional_costs: Decimal,
    pub docstatus: i32, // 0 = Draft, 1 = Submitted, 2 = Cancelled
    pub remarks: Option<String>,
}

impl StockEntry {
    /// Validates business invariants, purpose requirements, warehouse definitions, and recalculates values.
    pub fn validate(&mut self) -> Result<(), StockEntryError> {
        if self.items.is_empty() {
            return Err(StockEntryError::EmptyItems);
        }

        let mut outgoing = Decimal::ZERO;
        let mut incoming = Decimal::ZERO;

        let mut has_source_item = false;
        let mut has_target_item = false;

        for item in &mut self.items {
            if item.qty <= Decimal::ZERO {
                return Err(StockEntryError::InvalidQuantity {
                    item_code: item.item_code.clone(),
                    qty: item.qty,
                });
            }

            // Fill default warehouses if omitted at item level
            if self.purpose == StockEntryPurpose::Manufacture {
                if (item.is_finished_item || item.is_scrap_item)
                    && item.t_warehouse.is_none()
                    && self.to_warehouse.is_some()
                {
                    item.t_warehouse = self.to_warehouse.clone();
                } else if !item.is_finished_item
                    && !item.is_scrap_item
                    && item.s_warehouse.is_none()
                    && self.from_warehouse.is_some()
                {
                    item.s_warehouse = self.from_warehouse.clone();
                }
            } else {
                if item.s_warehouse.is_none() && self.from_warehouse.is_some() {
                    item.s_warehouse = self.from_warehouse.clone();
                }
                if item.t_warehouse.is_none() && self.to_warehouse.is_some() {
                    item.t_warehouse = self.to_warehouse.clone();
                }
            }

            match self.purpose {
                StockEntryPurpose::MaterialReceipt => {
                    if item.t_warehouse.is_none() {
                        return Err(StockEntryError::MissingTargetWarehouse(
                            item.item_code.clone(),
                        ));
                    }
                    if item.s_warehouse.is_some() {
                        return Err(StockEntryError::InvalidPurposeConfiguration {
                            purpose: self.purpose,
                            reason: format!(
                                "Item '{}' has source warehouse specified for Material Receipt",
                                item.item_code
                            ),
                        });
                    }
                }
                StockEntryPurpose::MaterialIssue => {
                    if item.s_warehouse.is_none() {
                        return Err(StockEntryError::MissingSourceWarehouse(
                            item.item_code.clone(),
                        ));
                    }
                    if item.t_warehouse.is_some() {
                        return Err(StockEntryError::InvalidPurposeConfiguration {
                            purpose: self.purpose,
                            reason: format!(
                                "Item '{}' has target warehouse specified for Material Issue",
                                item.item_code
                            ),
                        });
                    }
                }
                StockEntryPurpose::MaterialTransfer => {
                    let s_wh = item.s_warehouse.as_ref().ok_or_else(|| {
                        StockEntryError::MissingSourceWarehouse(item.item_code.clone())
                    })?;
                    let t_wh = item.t_warehouse.as_ref().ok_or_else(|| {
                        StockEntryError::MissingTargetWarehouse(item.item_code.clone())
                    })?;

                    if s_wh == t_wh {
                        return Err(StockEntryError::IdenticalSourceAndTargetWarehouse {
                            item_code: item.item_code.clone(),
                            warehouse: s_wh.clone(),
                        });
                    }
                }
                StockEntryPurpose::Manufacture | StockEntryPurpose::Repack => {
                    if item.s_warehouse.is_none() && item.t_warehouse.is_none() {
                        return Err(StockEntryError::InvalidPurposeConfiguration {
                            purpose: self.purpose,
                            reason: format!(
                                "Item '{}' must specify either source warehouse (raw material) or target warehouse (finished/scrap item)",
                                item.item_code
                            ),
                        });
                    }
                    if item.s_warehouse.is_some() && item.t_warehouse.is_some() {
                        return Err(StockEntryError::InvalidPurposeConfiguration {
                            purpose: self.purpose,
                            reason: format!(
                                "Item '{}' in Manufacture cannot have both source and target warehouse simultaneously",
                                item.item_code
                            ),
                        });
                    }
                }
            }

            if item.s_warehouse.is_some() {
                has_source_item = true;
            }
            if item.t_warehouse.is_some() {
                has_target_item = true;
            }

            // Recalculate amount
            item.amount = item.qty * item.basic_rate;

            if item.s_warehouse.is_some() && item.t_warehouse.is_none() {
                outgoing += item.amount;
            } else if item.t_warehouse.is_some() && item.s_warehouse.is_none() {
                incoming += item.amount;
            } else if item.s_warehouse.is_some() && item.t_warehouse.is_some() {
                // Transfer moves both
                outgoing += item.amount;
                incoming += item.amount;
            }
        }

        if matches!(
            self.purpose,
            StockEntryPurpose::Manufacture | StockEntryPurpose::Repack
        ) {
            if !has_source_item {
                return Err(StockEntryError::InvalidPurposeConfiguration {
                    purpose: self.purpose,
                    reason: "Manufacture entry must contain at least one raw material consumption line (source warehouse)".into(),
                });
            }
            if !has_target_item {
                return Err(StockEntryError::InvalidPurposeConfiguration {
                    purpose: self.purpose,
                    reason: "Manufacture entry must contain at least one finished good or scrap production line (target warehouse)".into(),
                });
            }
        }

        self.total_additional_costs = self.additional_costs.iter().map(|c| c.amount).sum();

        // For Manufacture: distribute raw materials + additional costs onto finished items
        if self.purpose == StockEntryPurpose::Manufacture {
            let total_manufacturing_cost = outgoing + self.total_additional_costs;
            let finished_count = self.items.iter().filter(|i| i.is_finished_item).count();

            if finished_count > 0 {
                let finished_qty_sum: Decimal = self
                    .items
                    .iter()
                    .filter(|i| i.is_finished_item)
                    .map(|i| i.qty)
                    .sum();

                if finished_qty_sum > Decimal::ZERO {
                    let unit_finished_rate = total_manufacturing_cost / finished_qty_sum;
                    incoming = Decimal::ZERO;

                    for item in &mut self.items {
                        if item.is_finished_item {
                            item.basic_rate = unit_finished_rate;
                            item.amount = item.qty * unit_finished_rate;
                        }
                        if item.t_warehouse.is_some() {
                            incoming += item.amount;
                        }
                    }
                }
            }
        }

        self.total_outgoing_value = outgoing;
        self.total_incoming_value = incoming;
        self.value_difference = incoming - outgoing;

        Ok(())
    }

    /// Verifies that no source warehouse goes into negative stock given an available stock lookup function.
    pub fn validate_stock_availability<F>(&self, mut get_stock: F) -> Result<(), StockEntryError>
    where
        F: FnMut(&str, &str) -> Decimal, // (item_code, warehouse) -> available_qty
    {
        for item in &self.items {
            if let Some(ref s_wh) = item.s_warehouse {
                let available = get_stock(item.item_code.as_str(), s_wh.as_str());
                if available < item.qty {
                    return Err(StockEntryError::NegativeStockDetected {
                        item_code: item.item_code.clone(),
                        warehouse: s_wh.clone(),
                        available,
                        required: item.qty,
                    });
                }
            }
        }
        Ok(())
    }

    /// Generates canonical Stock Ledger Entries (SLEs).
    /// Outward items create negative actual_qty; Inward items create positive actual_qty.
    pub fn generate_stock_ledger_entries(&self) -> Vec<StockLedgerEntry> {
        let mut sles = Vec::new();
        let voucher_type = "Stock Entry".to_string();
        let voucher_no = self.name.to_string();

        for (idx, item) in self.items.iter().enumerate() {
            // Outward movement
            if let Some(ref s_wh) = item.s_warehouse {
                sles.push(StockLedgerEntry {
                    name: format!("{}-s-{}", self.name, idx + 1),
                    item_code: item.item_code.to_string(),
                    warehouse: s_wh.to_string(),
                    actual_qty: -item.qty,
                    incoming_rate: item.basic_rate,
                    valuation_rate: item.basic_rate,
                    stock_value_difference: -item.amount,
                    posting_date: self.posting_date,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                });
            }

            // Inward movement
            if let Some(ref t_wh) = item.t_warehouse {
                sles.push(StockLedgerEntry {
                    name: format!("{}-t-{}", self.name, idx + 1),
                    item_code: item.item_code.to_string(),
                    warehouse: t_wh.to_string(),
                    actual_qty: item.qty,
                    incoming_rate: item.basic_rate,
                    valuation_rate: item.basic_rate,
                    stock_value_difference: item.amount,
                    posting_date: self.posting_date,
                    voucher_type: voucher_type.clone(),
                    voucher_no: voucher_no.clone(),
                });
            }
        }

        sles
    }

    /// Generates balanced double-entry General Ledger (GL) entries for this Stock Entry.
    ///
    /// Accounting rules:
    /// - Inward stock: Debit Target Warehouse `Stock In Hand` Asset.
    /// - Outward stock: Credit Source Warehouse `Stock In Hand` Asset.
    /// - Additional Costs: Credit the respective Operating/Labor Expense/Payable account.
    /// - Stock Adjustment / COGS: Balances any variance or net issue/receipt.
    pub fn generate_gl_entries<F>(
        &self,
        default_stock_adjustment_account: &str,
        mut get_warehouse_account: F,
    ) -> Result<Vec<GlEntry>, StockEntryError>
    where
        F: FnMut(&str) -> Result<CompactString, StockEntryError>,
    {
        let mut entries = Vec::new();
        let voucher_type = CompactString::new("Stock Entry");
        let voucher_no = self.name.clone();
        let company = self.company.clone();

        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        // 1. Process items
        for (idx, item) in self.items.iter().enumerate() {
            // Source warehouse: Stock is leaving -> Credit Stock In Hand
            if let Some(ref s_wh) = item.s_warehouse {
                let account = get_warehouse_account(s_wh.as_str())?;
                entries.push(GlEntry {
                    name: format!("{}-{}-out", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: account.to_string(),
                    debit: Decimal::ZERO,
                    credit: item.amount,
                    voucher_type: voucher_type.to_string(),
                    voucher_no: voucher_no.to_string(),
                    party_type: None,
                    party: None,
                    company: company.to_string(),
                });
                total_credit += item.amount;
            }

            // Target warehouse: Stock is entering -> Debit Stock In Hand
            if let Some(ref t_wh) = item.t_warehouse {
                let account = get_warehouse_account(t_wh.as_str())?;
                entries.push(GlEntry {
                    name: format!("{}-{}-in", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: account.to_string(),
                    debit: item.amount,
                    credit: Decimal::ZERO,
                    voucher_type: voucher_type.to_string(),
                    voucher_no: voucher_no.to_string(),
                    party_type: None,
                    party: None,
                    company: company.to_string(),
                });
                total_debit += item.amount;
            }
        }

        // 2. Process additional operating costs (e.g. Labor / Electricity consumed) -> Credit expense/liability
        for (idx, cost) in self.additional_costs.iter().enumerate() {
            if cost.amount > Decimal::ZERO {
                entries.push(GlEntry {
                    name: format!("{}-cost-{}", self.name, idx + 1),
                    posting_date: self.posting_date,
                    account: cost.expense_account.to_string(),
                    debit: Decimal::ZERO,
                    credit: cost.amount,
                    voucher_type: voucher_type.to_string(),
                    voucher_no: voucher_no.to_string(),
                    party_type: None,
                    party: None,
                    company: company.to_string(),
                });
                total_credit += cost.amount;
            }
        }

        // 3. Balance the entry with the Stock Adjustment / Expense account if imbalanced
        let diff = total_debit - total_credit;
        if diff != Decimal::ZERO {
            if diff > Decimal::ZERO {
                // Debits exceed credits: Need Credit to balancing account (e.g. Stock Adjustment income/opening)
                entries.push(GlEntry {
                    name: format!("{}-adj", self.name),
                    posting_date: self.posting_date,
                    account: default_stock_adjustment_account.to_string(),
                    debit: Decimal::ZERO,
                    credit: diff,
                    voucher_type: voucher_type.to_string(),
                    voucher_no: voucher_no.to_string(),
                    party_type: None,
                    party: None,
                    company: company.to_string(),
                });
                total_credit += diff;
            } else {
                // Credits exceed debits: Need Debit to balancing account (e.g. Stock Adjustment expense / consumption)
                let abs_diff = -diff;
                entries.push(GlEntry {
                    name: format!("{}-adj", self.name),
                    posting_date: self.posting_date,
                    account: default_stock_adjustment_account.to_string(),
                    debit: abs_diff,
                    credit: Decimal::ZERO,
                    voucher_type: voucher_type.to_string(),
                    voucher_no: voucher_no.to_string(),
                    party_type: None,
                    party: None,
                    company: company.to_string(),
                });
                total_debit += abs_diff;
            }
        }

        // Final verification of strict zero-drift balance
        if total_debit != total_credit {
            return Err(StockEntryError::UnbalancedGlEntries {
                total_debit,
                total_credit,
            });
        }

        Ok(entries)
    }

    /// Submits the Stock Entry, advancing docstatus to 1 (Submitted).
    pub fn submit(&mut self) -> Result<(), StockEntryError> {
        if self.docstatus != 0 {
            return Err(StockEntryError::InvalidDocStatus(self.docstatus));
        }
        self.validate()?;
        self.docstatus = 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_stock_entry_material_transfer_validation_and_sles() {
        let mut entry = StockEntry {
            name: "STE-2026-0001".into(),
            purpose: StockEntryPurpose::MaterialTransfer,
            posting_date: NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
            company: "Acme Corp".into(),
            from_warehouse: Some("Stores - ACME".into()),
            to_warehouse: Some("Finished Goods - ACME".into()),
            items: vec![StockEntryItem {
                item_code: "STEEL-ROD-01".into(),
                item_name: "Steel Rod 10mm".into(),
                s_warehouse: None, // inherited
                t_warehouse: None, // inherited
                qty: dec!(50.0),
                basic_rate: dec!(12.50),
                amount: Decimal::ZERO,
                serial_nos: vec![],
                batch_no: None,
                expense_account: None,
                cost_center: None,
                is_finished_item: false,
                is_scrap_item: false,
            }],
            additional_costs: vec![],
            total_incoming_value: Decimal::ZERO,
            total_outgoing_value: Decimal::ZERO,
            value_difference: Decimal::ZERO,
            total_additional_costs: Decimal::ZERO,
            docstatus: 0,
            remarks: Some("Internal warehouse transfer".into()),
        };

        entry.validate().expect("Transfer validation failed");
        assert_eq!(entry.items[0].s_warehouse.as_deref(), Some("Stores - ACME"));
        assert_eq!(
            entry.items[0].t_warehouse.as_deref(),
            Some("Finished Goods - ACME")
        );
        assert_eq!(entry.items[0].amount, dec!(625.00));
        assert_eq!(entry.total_outgoing_value, dec!(625.00));
        assert_eq!(entry.total_incoming_value, dec!(625.00));

        // Test stock availability check
        let availability_ok = entry.validate_stock_availability(|_item, _wh| dec!(100.0));
        assert!(availability_ok.is_ok());

        let availability_err = entry.validate_stock_availability(|_item, _wh| dec!(20.0));
        assert!(matches!(
            availability_err,
            Err(StockEntryError::NegativeStockDetected { .. })
        ));

        // Generate SLEs
        let sles = entry.generate_stock_ledger_entries();
        assert_eq!(sles.len(), 2);
        assert_eq!(sles[0].warehouse, "Stores - ACME");
        assert_eq!(sles[0].actual_qty, dec!(-50.0));
        assert_eq!(sles[0].stock_value_difference, dec!(-625.00));

        assert_eq!(sles[1].warehouse, "Finished Goods - ACME");
        assert_eq!(sles[1].actual_qty, dec!(50.0));
        assert_eq!(sles[1].stock_value_difference, dec!(625.00));

        // Generate GL entries
        let gl_entries = entry
            .generate_gl_entries("5110 - Stock Adjustment", |wh| match wh {
                "Stores - ACME" => Ok("1300 - Raw Materials".into()),
                "Finished Goods - ACME" => Ok("1310 - Finished Goods Inventory".into()),
                _ => Err(StockEntryError::WarehouseAccountNotFound(wh.into())),
            })
            .expect("GL entry generation failed");

        assert_eq!(gl_entries.len(), 2);
        let credit_entry = gl_entries
            .iter()
            .find(|e| e.credit > Decimal::ZERO)
            .unwrap();
        let debit_entry = gl_entries.iter().find(|e| e.debit > Decimal::ZERO).unwrap();

        assert_eq!(credit_entry.account, "1300 - Raw Materials");
        assert_eq!(credit_entry.credit, dec!(625.00));

        assert_eq!(debit_entry.account, "1310 - Finished Goods Inventory");
        assert_eq!(debit_entry.debit, dec!(625.00));
    }

    #[test]
    fn test_stock_entry_manufacture_with_additional_costs() {
        let mut entry = StockEntry {
            name: "MFG-2026-0001".into(),
            purpose: StockEntryPurpose::Manufacture,
            posting_date: NaiveDate::from_ymd_opt(2026, 10, 2).unwrap(),
            company: "Acme Corp".into(),
            from_warehouse: None,
            to_warehouse: None,
            items: vec![
                // Raw Material 1: 10 units @ $5 = $50
                StockEntryItem {
                    item_code: "RM-WOOD".into(),
                    item_name: "Treated Wood".into(),
                    s_warehouse: Some("Raw Materials - ACME".into()),
                    t_warehouse: None,
                    qty: dec!(10.0),
                    basic_rate: dec!(5.0),
                    amount: Decimal::ZERO,
                    serial_nos: vec![],
                    batch_no: None,
                    expense_account: None,
                    cost_center: None,
                    is_finished_item: false,
                    is_scrap_item: false,
                },
                // Raw Material 2: 20 units @ $2.5 = $50
                StockEntryItem {
                    item_code: "RM-SCREWS".into(),
                    item_name: "Steel Screws".into(),
                    s_warehouse: Some("Raw Materials - ACME".into()),
                    t_warehouse: None,
                    qty: dec!(20.0),
                    basic_rate: dec!(2.5),
                    amount: Decimal::ZERO,
                    serial_nos: vec![],
                    batch_no: None,
                    expense_account: None,
                    cost_center: None,
                    is_finished_item: false,
                    is_scrap_item: false,
                },
                // Finished Good: 2 Tables produced
                StockEntryItem {
                    item_code: "FG-TABLE".into(),
                    item_name: "Wooden Dining Table".into(),
                    s_warehouse: None,
                    t_warehouse: Some("Finished Goods - ACME".into()),
                    qty: dec!(2.0),
                    basic_rate: Decimal::ZERO, // auto-calculated
                    amount: Decimal::ZERO,
                    serial_nos: vec![],
                    batch_no: None,
                    expense_account: None,
                    cost_center: None,
                    is_finished_item: true,
                    is_scrap_item: false,
                },
            ],
            additional_costs: vec![
                StockEntryAdditionalCost {
                    expense_account: "5200 - Direct Labor".into(),
                    description: "Assembly Labor".into(),
                    amount: dec!(40.00),
                },
                StockEntryAdditionalCost {
                    expense_account: "5210 - Factory Power".into(),
                    description: "Machining Electricity".into(),
                    amount: dec!(10.00),
                },
            ],
            total_incoming_value: Decimal::ZERO,
            total_outgoing_value: Decimal::ZERO,
            value_difference: Decimal::ZERO,
            total_additional_costs: Decimal::ZERO,
            docstatus: 0,
            remarks: Some("Table assembly production batch".into()),
        };

        // Validate manufacture logic
        entry.validate().expect("Manufacture validation failed");

        // Total raw materials = 50 + 50 = 100.
        // Total additional costs = 40 + 10 = 50.
        // Total cost of FG = 150 across 2 tables -> $75/table.
        assert_eq!(entry.total_outgoing_value, dec!(100.00));
        assert_eq!(entry.total_additional_costs, dec!(50.00));
        assert_eq!(entry.total_incoming_value, dec!(150.00));
        assert_eq!(entry.items[2].basic_rate, dec!(75.00));
        assert_eq!(entry.items[2].amount, dec!(150.00));

        // Submit document
        entry.submit().expect("Document submission failed");
        assert_eq!(entry.docstatus, 1);

        // GL validation
        let gl_entries = entry
            .generate_gl_entries("5110 - Stock Adjustment", |wh| match wh {
                "Raw Materials - ACME" => Ok("1300 - Raw Materials Inventory".into()),
                "Finished Goods - ACME" => Ok("1310 - Finished Goods Inventory".into()),
                _ => Err(StockEntryError::WarehouseAccountNotFound(wh.into())),
            })
            .expect("GL entries should balance");

        let sum_debit: Decimal = gl_entries.iter().map(|e| e.debit).sum();
        let sum_credit: Decimal = gl_entries.iter().map(|e| e.credit).sum();
        assert_eq!(sum_debit, sum_credit);
        assert_eq!(sum_debit, dec!(150.00));
    }
}
