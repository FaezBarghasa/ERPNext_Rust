//! ASC 606 / IFRS 15 Multi-Step Revenue Recognition & RevOps Engine.

use chrono::Utc;
use erp_software::{
    Asc606Engine, CustomerContract as SoftwareCustomerContract,
    PerformanceObligation as SoftwarePerformanceObligation,
    RecognitionTiming as SoftwareRecognitionTiming,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SatisfactionMethod {
    PointInTime,                    // Goods receipt / delivery
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
    ///
    /// Delegated directly to [`erp_software::Asc606Engine`] to ensure mathematical consistency
    /// across software licensing and CRM contracts, specifically preserving the residual penny-rounding
    /// allocation invariant: $\sum_{i=1}^n \text{AllocatedPrice}_i \equiv \text{TransactionPrice}$.
    pub fn allocate_contract_price(contract: &mut RevenueContract) -> Result<(), String> {
        if contract.obligations.is_empty() {
            return Err("Contract has no performance obligations".into());
        }

        let total_ssp: Decimal = contract
            .obligations
            .iter()
            .map(|p| p.standalone_selling_price)
            .sum();

        if total_ssp.is_zero() {
            return Err("Total Standalone Selling Price (SSP) cannot be zero".into());
        }

        let software_contract = Self::to_customer_contract(contract);
        let allocated = Asc606Engine::allocate_transaction_price(&software_contract)
            .map_err(|e| e.to_string())?;

        for (pob, alloc) in contract.obligations.iter_mut().zip(allocated) {
            pob.allocated_price = alloc.allocated_price;
        }

        Ok(())
    }

    /// Converts a [`RevenueContract`] into a canonical [`erp_software::CustomerContract`].
    #[must_use]
    pub fn to_customer_contract(contract: &RevenueContract) -> SoftwareCustomerContract {
        let now = Utc::now();
        let obligations: Vec<SoftwarePerformanceObligation> = contract
            .obligations
            .iter()
            .map(|pob| SoftwarePerformanceObligation {
                id: pob.id.clone(),
                name: pob.description.clone(),
                standalone_selling_price: pob.standalone_selling_price,
                timing: match pob.method {
                    SatisfactionMethod::PointInTime => SoftwareRecognitionTiming::PointInTime(now),
                    SatisfactionMethod::OverTimePercentageOfCompletion => {
                        SoftwareRecognitionTiming::OverTimeMilestone {
                            milestone_weights: vec![rust_decimal_macros::dec!(100)],
                        }
                    }
                },
                deferred_revenue_account: "2200 - Deferred Revenue".into(),
                recognized_revenue_account: "4100 - Software Revenue".into(),
            })
            .collect();

        SoftwareCustomerContract {
            contract_id: contract.contract_id.clone(),
            customer_id: contract.customer_id.clone(),
            transaction_price: contract.transaction_price,
            receivable_account: "1200 - Accounts Receivable".into(),
            start_date: now,
            created_at: now,
            obligations,
        }
    }

    /// Progressively recognizes revenue based on actual milestone completion percentage.
    pub fn recognize_progress(
        pob: &mut PerformanceObligation,
        completion_percent: Decimal,
    ) -> Decimal {
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

    #[test]
    fn test_asc606_residual_penny_rounding_invariant() {
        // 3 equal obligations of SSP $100 against transaction price $100.
        // Under naive division: 33.33 + 33.33 + 33.33 = 99.99 (drift of 1 penny).
        // Under Asc606Engine residual allocation: 33.33 + 33.33 + 33.34 = 100.00 (invariant preserved).
        let mut contract = RevenueContract {
            contract_id: "REV-PENNY-001".into(),
            customer_id: "Trio Client".into(),
            transaction_price: dec!(100.00),
            obligations: vec![
                PerformanceObligation {
                    id: "POB-A".into(),
                    description: "Part A".into(),
                    standalone_selling_price: dec!(100.00),
                    allocated_price: dec!(0),
                    method: SatisfactionMethod::PointInTime,
                    recognized_revenue: dec!(0),
                    is_satisfied: false,
                },
                PerformanceObligation {
                    id: "POB-B".into(),
                    description: "Part B".into(),
                    standalone_selling_price: dec!(100.00),
                    allocated_price: dec!(0),
                    method: SatisfactionMethod::PointInTime,
                    recognized_revenue: dec!(0),
                    is_satisfied: false,
                },
                PerformanceObligation {
                    id: "POB-C".into(),
                    description: "Part C".into(),
                    standalone_selling_price: dec!(100.00),
                    allocated_price: dec!(0),
                    method: SatisfactionMethod::PointInTime,
                    recognized_revenue: dec!(0),
                    is_satisfied: false,
                },
            ],
            audit_hash: "hash_trio".into(),
        };

        RevOpsEngine::allocate_contract_price(&mut contract).unwrap();
        assert_eq!(contract.obligations[0].allocated_price, dec!(33.33));
        assert_eq!(contract.obligations[1].allocated_price, dec!(33.33));
        assert_eq!(contract.obligations[2].allocated_price, dec!(33.34));

        let total_allocated: Decimal = contract.obligations.iter().map(|p| p.allocated_price).sum();
        assert_eq!(total_allocated, contract.transaction_price);
    }

    #[test]
    fn test_to_customer_contract_and_amortization_schedule() {
        let contract = RevenueContract {
            contract_id: "REV-AMORT-01".into(),
            customer_id: "Enterprise Corp".into(),
            transaction_price: dec!(50000.00),
            obligations: vec![PerformanceObligation {
                id: "POB-SUB".into(),
                description: "SaaS Annual Subscription".into(),
                standalone_selling_price: dec!(50000.00),
                allocated_price: dec!(50000.00),
                method: SatisfactionMethod::OverTimePercentageOfCompletion,
                recognized_revenue: dec!(0),
                is_satisfied: false,
            }],
            audit_hash: "hash_sub".into(),
        };

        let software_contract = RevOpsEngine::to_customer_contract(&contract);
        assert_eq!(software_contract.contract_id, "REV-AMORT-01");
        assert_eq!(software_contract.transaction_price, dec!(50000.00));
        assert_eq!(software_contract.obligations.len(), 1);

        let allocated = Asc606Engine::allocate_transaction_price(&software_contract)
            .expect("Allocate transaction price");
        assert_eq!(allocated[0].allocated_price, dec!(50000.00));

        let schedules = Asc606Engine::generate_amortization_schedules(
            &software_contract.contract_id,
            &allocated,
        )
        .expect("Generate amortization schedules");

        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].recognized_amount, dec!(50000.00));

        let gl_lines = Asc606Engine::build_amortization_gl_lines(&schedules[0]);
        assert_eq!(gl_lines.len(), 2);
        assert_eq!(gl_lines[0].account, "2200 - Deferred Revenue");
        assert_eq!(gl_lines[0].debit, dec!(50000.00));
        assert_eq!(gl_lines[1].account, "4100 - Software Revenue");
        assert_eq!(gl_lines[1].credit, dec!(50000.00));
    }
}
