use crate::landed_cost::distribute_landed_cost;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Item supplied by customer for inward job-work subcontracting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomerSuppliedItem {
    /// Item code of the supplied raw material/component.
    pub item_code: String,
    /// Batch or serial number assigned by customer.
    pub batch_no: Option<String>,
    /// Received quantity.
    pub qty_received: Decimal,
    /// Consumed quantity in production.
    pub qty_consumed: Decimal,
}

/// Service line charge on inward subcontracted job-work.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractServiceCharge {
    /// Service item code (e.g. "JOB-WORK-COATING").
    pub service_item_code: String,
    /// Service quantity.
    pub qty: Decimal,
    /// Unit service rate.
    pub rate: Decimal,
    /// Total service amount (`qty * rate`).
    pub amount: Decimal,
}

/// Inward Subcontracting document tracking customer-supplied materials and job-work processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractedSalesOrder {
    /// Document identifier.
    pub name: String,
    /// Customer ID.
    pub customer: String,
    /// Dedicated customer warehouse location holding customer-owned stock.
    pub customer_warehouse: String,
    /// List of customer supplied components.
    pub supplied_items: Vec<CustomerSuppliedItem>,
    /// Service charges invoiced to the customer.
    pub service_charges: Vec<SubcontractServiceCharge>,
}

impl SubcontractedSalesOrder {
    /// Calculates total service charge to be billed to the customer (excluding cost of customer's own raw materials).
    pub fn calculate_billable_total(&self) -> Decimal {
        self.service_charges.iter().map(|s| s.amount).sum()
    }
}

/// Outward Subcontracting Receipt for finished goods received from a third-party subcontractor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubcontractingReceipt {
    /// Receipt ID.
    pub name: String,
    /// Subcontractor supplier ID.
    pub supplier: String,
    /// Finished good item code received.
    pub fg_item_code: String,
    /// Received finished good quantity.
    pub fg_qty: Decimal,
    /// Service processing rate per FG unit charged by subcontractor.
    pub service_rate: Decimal,
    /// Total value of raw materials consumed from subcontracting warehouse.
    pub raw_material_cost: Decimal,
    /// Additional landed charges (customs, freight, handling).
    pub additional_landed_cost: Decimal,
    /// Posting date.
    pub posting_date: NaiveDate,
}

impl SubcontractingReceipt {
    /// Computes the exact capitalized unit valuation rate for the received finished good:
    /// `Unit Valuation Rate = (Raw Material Cost + Service Charges + Landed Cost) / FG Qty`
    pub fn calculate_fg_valuation_rate(&self) -> Decimal {
        if self.fg_qty <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let total_service_cost = self.fg_qty * self.service_rate;
        let total_capitalized_cost =
            self.raw_material_cost + total_service_cost + self.additional_landed_cost;

        total_capitalized_cost / self.fg_qty
    }
}

/// Landed Cost Voucher allocator for multi-item subcontracting or purchase receipts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LandedCostVoucher {
    /// Voucher identifier.
    pub voucher_no: String,
    /// Total landed expenses to distribute.
    pub total_landed_cost: Decimal,
    /// Base item valuation amounts.
    pub receipt_item_values: Vec<Decimal>,
}

impl LandedCostVoucher {
    /// Distributes landed cost charges across items based on their base values.
    pub fn allocate_charges(&self) -> Vec<Decimal> {
        distribute_landed_cost(&self.receipt_item_values, self.total_landed_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_inward_subcontracting_service_total() {
        let sso = SubcontractedSalesOrder {
            name: "SSO-2026-01".into(),
            customer: "CUST-AIRCRAFT".into(),
            customer_warehouse: "Customer Stores - AIRCRAFT".into(),
            supplied_items: vec![CustomerSuppliedItem {
                item_code: "RAW-TITANIUM-BAR".into(),
                batch_no: Some("CUST-BATCH-99".into()),
                qty_received: dec!(100),
                qty_consumed: dec!(100),
            }],
            service_charges: vec![
                SubcontractServiceCharge {
                    service_item_code: "CNC-MILLING-SERVICE".into(),
                    qty: dec!(10),
                    rate: dec!(150.0),
                    amount: dec!(1500.0),
                },
                SubcontractServiceCharge {
                    service_item_code: "ANODIZING-SERVICE".into(),
                    qty: dec!(10),
                    rate: dec!(30.0),
                    amount: dec!(300.0),
                },
            ],
        };

        assert_eq!(sso.calculate_billable_total(), dec!(1800.0));
    }

    #[test]
    fn test_outward_subcontracting_fg_valuation() {
        let receipt = SubcontractingReceipt {
            name: "SCR-2026-001".into(),
            supplier: "SUPP-PLATING-CO".into(),
            fg_item_code: "GOLD-PLATED-CONNECTOR".into(),
            fg_qty: dec!(100),
            service_rate: dec!(5.0),             // $500 service
            raw_material_cost: dec!(2000.0),     // $2000 raw material consumed
            additional_landed_cost: dec!(200.0), // $200 courier/customs
            posting_date: NaiveDate::from_ymd_opt(2026, 9, 28).unwrap(),
        };

        // Total = 2000 + 500 + 200 = 2700 / 100 = $27.00 per unit
        assert_eq!(receipt.calculate_fg_valuation_rate(), dec!(27.0));
    }
}
