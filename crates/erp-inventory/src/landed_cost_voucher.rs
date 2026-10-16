//! Landed Cost Voucher Engine (`erp-inventory::landed_cost_voucher`).
//!
//! Allocates additional inbound import duties, ocean/air freight charges, and handling costs
//! to received stock items in Purchase Receipts or Subcontracting Receipts, updating inventory
//! valuation layers without altering purchase vendor payables.

use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Basis for distributing landed cost charges across received items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandedCostAllocationMethod {
    ByQty,
    ByAmount,
}

/// An individual item receipt row eligible for landed cost allocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LandedCostItem {
    pub receipt_document_type: CompactString, // "Purchase Receipt"
    pub receipt_document: CompactString,      // "PR-001"
    pub item_code: CompactString,
    pub qty: Decimal,
    pub original_rate: Decimal,
    pub original_amount: Decimal,
    pub allocated_charges: Decimal,
    pub landed_rate: Decimal,
}

/// An individual landed cost charge line item (freight, customs, insurance).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LandedCostCharge {
    pub expense_account: CompactString,
    pub description: CompactString,
    pub amount: Decimal,
}

/// Landed Cost Voucher Document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LandedCostVoucher {
    pub name: CompactString,
    pub company: CompactString,
    pub allocation_method: LandedCostAllocationMethod,
    pub charges: Vec<LandedCostCharge>,
    pub items: Vec<LandedCostItem>,
}

impl LandedCostVoucher {
    /// Distributes total charges proportionally across received items based on selected allocation method.
    pub fn allocate_charges(&mut self) -> Result<(), &'static str> {
        if self.items.is_empty() {
            return Err("No items present for landed cost allocation");
        }

        let total_charges: Decimal = self.charges.iter().map(|c| c.amount).sum();
        if total_charges <= Decimal::ZERO {
            return Ok(());
        }

        match self.allocation_method {
            LandedCostAllocationMethod::ByQty => {
                let total_qty: Decimal = self.items.iter().map(|item| item.qty).sum();
                if total_qty.is_zero() {
                    return Err("Total quantity cannot be zero for ByQty allocation");
                }

                for item in &mut self.items {
                    let share = (item.qty * total_charges) / total_qty;
                    item.allocated_charges = share;
                    item.landed_rate = item.original_rate + (share / item.qty);
                }
            }
            LandedCostAllocationMethod::ByAmount => {
                let total_amt: Decimal = self.items.iter().map(|item| item.original_amount).sum();
                if total_amt.is_zero() {
                    return Err("Total amount cannot be zero for ByAmount allocation");
                }

                for item in &mut self.items {
                    let share = (item.original_amount * total_charges) / total_amt;
                    item.allocated_charges = share;
                    item.landed_rate = (item.original_amount + share) / item.qty;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_landed_cost_allocation_by_amount() {
        let mut voucher = LandedCostVoucher {
            name: "LCV-001".into(),
            company: "Acme Imports".into(),
            allocation_method: LandedCostAllocationMethod::ByAmount,
            charges: vec![
                LandedCostCharge {
                    expense_account: "5120 - Customs Duty".into(),
                    description: "Import Tariff 10%".into(),
                    amount: dec!(300.0),
                },
                LandedCostCharge {
                    expense_account: "5130 - Ocean Freight".into(),
                    description: "Container Freight".into(),
                    amount: dec!(200.0),
                },
            ], // Total charges = $500
            items: vec![
                LandedCostItem {
                    receipt_document_type: "Purchase Receipt".into(),
                    receipt_document: "PR-1".into(),
                    item_code: "LAPTOP".into(),
                    qty: dec!(2.0),
                    original_rate: dec!(1000.0),
                    original_amount: dec!(2000.0), // 80% of total value ($2,000 / $2,500)
                    allocated_charges: Decimal::ZERO,
                    landed_rate: Decimal::ZERO,
                },
                LandedCostItem {
                    receipt_document_type: "Purchase Receipt".into(),
                    receipt_document: "PR-1".into(),
                    item_code: "MOUSE".into(),
                    qty: dec!(10.0),
                    original_rate: dec!(50.0),
                    original_amount: dec!(500.0), // 20% of total value ($500 / $2,500)
                    allocated_charges: Decimal::ZERO,
                    landed_rate: Decimal::ZERO,
                },
            ],
        };

        voucher.allocate_charges().unwrap();

        // 80% of $500 = $400 for Laptops ($200 per laptop -> Landed Rate = $1,200)
        assert_eq!(voucher.items[0].allocated_charges, dec!(400.0));
        assert_eq!(voucher.items[0].landed_rate, dec!(1200.0));

        // 20% of $500 = $100 for Mice ($10 per mouse -> Landed Rate = $60)
        assert_eq!(voucher.items[1].allocated_charges, dec!(100.0));
        assert_eq!(voucher.items[1].landed_rate, dec!(60.0));
    }
}
