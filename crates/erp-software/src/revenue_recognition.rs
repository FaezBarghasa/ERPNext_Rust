//! ASC 606 & IFRS 15 5-Step Revenue Recognition Engine.
//!
//! Provides mathematically rigorous allocation of transaction prices across multi-element
//! performance obligations (POBs) via relative Standalone Selling Prices (SSP) and generates
//! balanced General Ledger amortization posting lines without float drift.

use chrono::{DateTime, Utc};
use erp_accounting::decimal_ledger::DecLine;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::errors::SoftwareBillingError;

/// Timing mechanism for revenue satisfaction according to ASC 606-10-25-27.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecognitionTiming {
    /// Revenue recognized entirely at a distinct point in time upon delivery/activation.
    PointInTime(DateTime<Utc>),
    /// Revenue recognized straight-line over a duration across N equal accounting periods.
    OverTimeStraightLine {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        periods_count: usize,
    },
    /// Revenue recognized upon achievement of defined progress milestones.
    OverTimeMilestone {
        milestone_weights: Vec<Decimal>, // Must sum to 100%
    },
}

/// Performance Obligation (POB) representing a distinct promised good or service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceObligation {
    pub id: String,
    pub name: String,
    /// Standalone Selling Price (SSP) observable or estimated under ASC 606.
    pub standalone_selling_price: Decimal,
    pub timing: RecognitionTiming,
    pub deferred_revenue_account: String,
    pub recognized_revenue_account: String,
}

/// Allocated Performance Obligation with exact allocated transaction consideration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocatedObligation {
    pub obligation: PerformanceObligation,
    /// Exact allocated portion of the overall contract transaction price.
    pub allocated_price: Decimal,
}

/// Customer software/services contract subject to ASC 606 evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomerContract {
    pub contract_id: String,
    pub customer_id: String,
    /// Step 3: Total fixed and determinable consideration.
    pub transaction_price: Decimal,
    pub receivable_account: String,
    pub start_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// Step 2: Identified distinct performance obligations.
    pub obligations: Vec<PerformanceObligation>,
}

/// An individual periodic entry in a deferred revenue amortization schedule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AmortizationScheduleEntry {
    pub entry_id: String,
    pub contract_id: String,
    pub obligation_id: String,
    pub period_index: usize,
    pub period_date: DateTime<Utc>,
    pub recognized_amount: Decimal,
    pub cumulative_recognized: Decimal,
    pub remaining_deferred: Decimal,
    pub deferred_account: String,
    pub revenue_account: String,
}

/// Complete ASC 606 revenue recognition engine.
pub struct Asc606Engine;

impl Asc606Engine {
    /// Step 4: Allocates total contract transaction price across obligations based on relative SSP.
    ///
    /// Residual penny-rounding adjustment is allocated to the final obligation to ensure
    /// the exact algebraic invariant: $\sum \text{Allocated}_i \equiv \text{Transaction Price}$.
    pub fn allocate_transaction_price(
        contract: &CustomerContract,
    ) -> Result<Vec<AllocatedObligation>, SoftwareBillingError> {
        if contract.obligations.is_empty() {
            return Err(SoftwareBillingError::InvalidContractTerms(
                "Contract has no performance obligations".into(),
            ));
        }

        let total_ssp: Decimal = contract
            .obligations
            .iter()
            .fold(Decimal::ZERO, |acc, pob| acc + pob.standalone_selling_price);

        if total_ssp <= Decimal::ZERO {
            return Err(SoftwareBillingError::ZeroSellingPrice);
        }

        let mut allocated = Vec::with_capacity(contract.obligations.len());
        let mut cumulative_allocated = Decimal::ZERO;
        let last_idx = contract.obligations.len() - 1;

        for (idx, pob) in contract.obligations.iter().enumerate() {
            let allocated_price = if idx == last_idx {
                // Residual assignment guarantees exact match down to 0.00000000
                contract.transaction_price - cumulative_allocated
            } else {
                let ratio = pob.standalone_selling_price / total_ssp;
                let share = (contract.transaction_price * ratio).round_dp(2);
                cumulative_allocated += share;
                share
            };

            allocated.push(AllocatedObligation {
                obligation: pob.clone(),
                allocated_price,
            });
        }

        let final_sum = allocated
            .iter()
            .fold(Decimal::ZERO, |acc, a| acc + a.allocated_price);

        if final_sum != contract.transaction_price {
            return Err(SoftwareBillingError::SspAllocationMismatch {
                allocated: final_sum.to_string(),
                expected: contract.transaction_price.to_string(),
            });
        }

        Ok(allocated)
    }

    /// Step 5: Generates detailed periodic amortization schedules for all allocated obligations.
    pub fn generate_amortization_schedules(
        contract_id: &str,
        allocated_obligations: &[AllocatedObligation],
    ) -> Result<Vec<AmortizationScheduleEntry>, SoftwareBillingError> {
        let mut schedule = Vec::new();

        for allocated in allocated_obligations {
            let pob = &allocated.obligation;
            let total_amount = allocated.allocated_price;

            match &pob.timing {
                RecognitionTiming::PointInTime(date) => {
                    schedule.push(AmortizationScheduleEntry {
                        entry_id: format!("{}-{}-p1", contract_id, pob.id),
                        contract_id: contract_id.to_string(),
                        obligation_id: pob.id.clone(),
                        period_index: 1,
                        period_date: *date,
                        recognized_amount: total_amount,
                        cumulative_recognized: total_amount,
                        remaining_deferred: Decimal::ZERO,
                        deferred_account: pob.deferred_revenue_account.clone(),
                        revenue_account: pob.recognized_revenue_account.clone(),
                    });
                }
                RecognitionTiming::OverTimeStraightLine {
                    start,
                    end,
                    periods_count,
                } => {
                    if start >= end || *periods_count == 0 {
                        return Err(SoftwareBillingError::ScheduleOutOfPeriod {
                            start: start.to_string(),
                            end: end.to_string(),
                        });
                    }

                    let n = *periods_count;
                    let n_dec = Decimal::from(n);
                    let base_per_period = (total_amount / n_dec).round_dp(2);
                    let duration_per_period = (*end - *start) / (n as i32);

                    let mut cumulative = Decimal::ZERO;

                    for i in 1..=n {
                        let is_last = i == n;
                        let recognized_amount = if is_last {
                            total_amount - cumulative
                        } else {
                            base_per_period
                        };

                        cumulative += recognized_amount;
                        let remaining = total_amount - cumulative;
                        let period_date = *start + duration_per_period * (i as i32);

                        schedule.push(AmortizationScheduleEntry {
                            entry_id: format!("{}-{}-p{}", contract_id, pob.id, i),
                            contract_id: contract_id.to_string(),
                            obligation_id: pob.id.clone(),
                            period_index: i,
                            period_date,
                            recognized_amount,
                            cumulative_recognized: cumulative,
                            remaining_deferred: remaining,
                            deferred_account: pob.deferred_revenue_account.clone(),
                            revenue_account: pob.recognized_revenue_account.clone(),
                        });
                    }
                }
                RecognitionTiming::OverTimeMilestone { milestone_weights } => {
                    let total_weight: Decimal = milestone_weights.iter().copied().sum();
                    if total_weight != dec!(100) {
                        return Err(SoftwareBillingError::InvalidContractTerms(format!(
                            "Milestone weights must sum to 100%, found {}",
                            total_weight
                        )));
                    }

                    let mut cumulative = Decimal::ZERO;
                    let last_idx = milestone_weights.len() - 1;

                    for (idx, weight) in milestone_weights.iter().enumerate() {
                        let is_last = idx == last_idx;
                        let recognized_amount = if is_last {
                            total_amount - cumulative
                        } else {
                            (total_amount * (*weight / dec!(100))).round_dp(2)
                        };

                        cumulative += recognized_amount;
                        let remaining = total_amount - cumulative;

                        schedule.push(AmortizationScheduleEntry {
                            entry_id: format!("{}-{}-m{}", contract_id, pob.id, idx + 1),
                            contract_id: contract_id.to_string(),
                            obligation_id: pob.id.clone(),
                            period_index: idx + 1,
                            period_date: Utc::now(),
                            recognized_amount,
                            cumulative_recognized: cumulative,
                            remaining_deferred: remaining,
                            deferred_account: pob.deferred_revenue_account.clone(),
                            revenue_account: pob.recognized_revenue_account.clone(),
                        });
                    }
                }
            }
        }

        Ok(schedule)
    }

    /// Emits initial contract General Ledger lines:
    /// Debit: Accounts Receivable (Asset) = Transaction Price
    /// Credit: Deferred Revenue (Liability) per obligation = Allocated Price
    pub fn build_initial_contract_gl_lines(
        contract: &CustomerContract,
        allocated_obligations: &[AllocatedObligation],
    ) -> Vec<DecLine> {
        let mut lines = Vec::new();

        // Debit Accounts Receivable
        lines.push(DecLine {
            account: contract.receivable_account.clone(),
            debit: contract.transaction_price,
            credit: Decimal::ZERO,
        });

        // Credit Deferred Revenue Liability accounts
        for item in allocated_obligations {
            lines.push(DecLine {
                account: item.obligation.deferred_revenue_account.clone(),
                debit: Decimal::ZERO,
                credit: item.allocated_price,
            });
        }

        lines
    }

    /// Emits monthly / periodic deferred revenue amortization GL lines:
    /// Debit: Deferred Revenue (Liability reduction) = Recognized Amount
    /// Credit: Recognized Revenue (Revenue increase) = Recognized Amount
    pub fn build_amortization_gl_lines(entry: &AmortizationScheduleEntry) -> Vec<DecLine> {
        vec![
            DecLine {
                account: entry.deferred_account.clone(),
                debit: entry.recognized_amount,
                credit: Decimal::ZERO,
            },
            DecLine {
                account: entry.revenue_account.clone(),
                debit: Decimal::ZERO,
                credit: entry.recognized_amount,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use erp_accounting::decimal_ledger::verify_balanced_dec;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asc606_relative_ssp_allocation() {
        let now = Utc::now();
        // Bundle: Software License ($12,000 SSP) + Implementation Service ($8,000 SSP)
        // Bundle Discounted Transaction Price = $16,000 (20% bundle discount)
        // License should be allocated: 16000 * (12000 / 20000) = $9,600
        // Service should be allocated: 16000 * (8000 / 20000) = $6,400
        let contract = CustomerContract {
            contract_id: "CTR-2026-001".into(),
            customer_id: "CUST-ACME".into(),
            transaction_price: dec!(16000.00),
            receivable_account: "1100-AR".into(),
            start_date: now,
            created_at: now,
            obligations: vec![
                PerformanceObligation {
                    id: "pob-lic".into(),
                    name: "SaaS Enterprise Term License".into(),
                    standalone_selling_price: dec!(12000.00),
                    timing: RecognitionTiming::OverTimeStraightLine {
                        start: now,
                        end: now + Duration::days(360),
                        periods_count: 12,
                    },
                    deferred_revenue_account: "2100-Deferred-SaaS".into(),
                    recognized_revenue_account: "4000-SaaS-Rev".into(),
                },
                PerformanceObligation {
                    id: "pob-impl".into(),
                    name: "Professional Onboarding Services".into(),
                    standalone_selling_price: dec!(8000.00),
                    timing: RecognitionTiming::PointInTime(now + Duration::days(30)),
                    deferred_revenue_account: "2110-Deferred-Services".into(),
                    recognized_revenue_account: "4100-Services-Rev".into(),
                },
            ],
        };

        let allocated = Asc606Engine::allocate_transaction_price(&contract).unwrap();
        assert_eq!(allocated.len(), 2);
        assert_eq!(allocated[0].allocated_price, dec!(9600.00));
        assert_eq!(allocated[1].allocated_price, dec!(6400.00));

        // Test Initial GL lines
        let initial_gl = Asc606Engine::build_initial_contract_gl_lines(&contract, &allocated);
        assert!(verify_balanced_dec(&initial_gl));

        // Test Schedule generation
        let schedules =
            Asc606Engine::generate_amortization_schedules(&contract.contract_id, &allocated)
                .unwrap();
        // 12 monthly straight-line periods + 1 point-in-time service entry = 13 entries
        assert_eq!(schedules.len(), 13);

        // Verify monthly straight-line recognized amount: 9600 / 12 = 800.00 per month
        assert_eq!(schedules[0].recognized_amount, dec!(800.00));
        assert_eq!(schedules[11].cumulative_recognized, dec!(9600.00));
        assert_eq!(schedules[11].remaining_deferred, dec!(0.00));

        // Test Amortization GL line balance for month 1
        let month1_gl = Asc606Engine::build_amortization_gl_lines(&schedules[0]);
        assert!(verify_balanced_dec(&month1_gl));
    }
}
