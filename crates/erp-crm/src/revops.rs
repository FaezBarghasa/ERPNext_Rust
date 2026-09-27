//! ASC 606 / IFRS 15 Multi-Step Revenue Recognition & RevOps Engine.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SatisfactionMethod {
    PointInTime, // Goods receipt / delivery
    OverTimePercentageOfCompletion, // Timesheet hours / milestone burn
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceObligation {
    pub id: String,
    pub description: String,
    pub standalone_selling_price: Decimal, // SSP
    pub allocated_price: Decimal,          // Allocated Transaction Price
    pub method: SatisfactionMethod,
    pub recognized_revenue: Decimal,
    pub is_satisfied: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueContract {
    pub contract_id: String,
    pub customer_id: String,
    pub transaction_price: Decimal, // TP
    pub obligations: Vec<PerformanceObligation>,
    pub audit_hash: String,
}

pub struct RevOpsEngine;

impl RevOpsEngine {
    /// 5-Step Revenue Recognition: Allocates Transaction Price (TP) based on relative Standalone Selling Prices (SSP).
    pub fn allocate_contract_price(contract: &mut RevenueContract) -> Result<(), String> {
        let total_ssp: Decimal = contract
            .obligations
            .iter()
            .map(|p| p.standalone_selling_price)
            .sum();

        if total_ssp.is_zero() {
            return Err("Total Standalone Selling Price (SSP) cannot be zero".into());
        }

        for pob in &mut contract.obligations {
            let ratio = pob.standalone_selling_price / total_ssp;
            pob.allocated_price = (contract.transaction_price * ratio).round_dp(2);
        }

        Ok(())
    }

    /// Progressively recognizes revenue based on actual milestone completion percentage.
    pub fn recognize_progress(pob: &mut PerformanceObligation, completion_percent: Decimal) -> Decimal {
        let target_revenue = (pob.allocated_price * completion_percent).round_dp(2);
        let newly_recognized = target_revenue - pob.recognized_revenue;
        pob.recognized_revenue = target_revenue;
        if completion_percent >= Decimal::ONE {
            pob.is_satisfied = true;
        }
        newly_recognized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asc606_ssp_allocation_and_recognition() {
        let mut contract = RevenueContract {
            contract_id: "REV-2026-001".into(),
            customer_id: "Acme Corp".into(),
            transaction_price: dec!(120000), // Discounted bundle deal (Total SSP is 150,000)
            obligations: vec![
                PerformanceObligation {
                    id: "POB-1".into(),
                    description: "Software Perpetual License".into(),
                    standalone_selling_price: dec!(90000), // 60% of SSP -> 72,000 allocated
                    allocated_price: dec!(0),
                    method: SatisfactionMethod::PointInTime,
                    recognized_revenue: dec!(0),
                    is_satisfied: false,
                },
                PerformanceObligation {
                    id: "POB-2".into(),
                    description: "Implementation Services".into(),
                    standalone_selling_price: dec!(60000), // 40% of SSP -> 48,000 allocated
                    allocated_price: dec!(0),
                    method: SatisfactionMethod::OverTimePercentageOfCompletion,
                    recognized_revenue: dec!(0),
                    is_satisfied: false,
                },
            ],
            audit_hash: "hash123".into(),
        };

        RevOpsEngine::allocate_contract_price(&mut contract).unwrap();
        assert_eq!(contract.obligations[0].allocated_price, dec!(72000.00));
        assert_eq!(contract.obligations[1].allocated_price, dec!(48000.00));

        let newly_rec = RevOpsEngine::recognize_progress(&mut contract.obligations[1], dec!(0.50));
        assert_eq!(newly_rec, dec!(24000.00));
        assert_eq!(contract.obligations[1].recognized_revenue, dec!(24000.00));
    }
}
