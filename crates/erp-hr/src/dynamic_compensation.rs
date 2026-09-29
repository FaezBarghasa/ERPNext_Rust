//! Precision Workforce Time-Tracking & Dynamic Compensation Engine (`erp_hr::dynamic_compensation`).
//!
//! Implements Pillar XXIV: Hybrid compensation models combining tiered hourly rates,
//! shift differentials (night +35%, holiday 2.0x), piece-rate manufacturing (کارمزدی)
//! with scrap penalties, milestone bonuses, and hybrid floor guarantees.

use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Job card production yield for piece-rate compensation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobCardPieceworkLog {
    pub job_card_id: CompactString,
    pub operation: CompactString,
    pub qty_accepted: Decimal,
    pub rate_per_unit: Decimal,
    pub qty_scrap: Decimal,
    pub scrap_penalty_rate: Decimal,
}

/// Project / task milestone completion log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilestoneBonusLog {
    pub project: CompactString,
    pub task_name: CompactString,
    pub bonus_amount: Decimal,
    pub qa_approved: bool,
}

/// Time-tracking hours breakdown for a worker shift/period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShiftHoursBreakdown {
    pub regular_hours: Decimal,
    pub overtime_hours: Decimal,
    pub holiday_hours: Decimal,
    pub night_shift_hours: Decimal,
    pub base_hourly_rate: Decimal,
}

/// Selected compensation plan for employee/worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompensationModelType {
    TieredHourly,
    PurePieceRate,
    ProjectMilestone,
    HybridGuaranteedMinimum,
}

/// Summary of computed dynamic compensation payout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicCompensationResult {
    pub worker_id: CompactString,
    pub model_used: CompensationModelType,
    pub time_based_earned: Decimal,
    pub piece_rate_earned: Decimal,
    pub milestone_bonus_earned: Decimal,
    pub guaranteed_minimum: Decimal,
    pub gross_payable: Decimal,
    pub cost_center: CompactString,
}

/// Compensation calculation controller.
pub struct DynamicCompensationEngine;

impl DynamicCompensationEngine {
    /// Computes Tiered Time & Overtime Multipliers:
    /// W_time = h_reg * r_base + h_ot * (1.4 * r_base) + h_hol * (2.0 * r_base) + h_night * (1.35 * r_base)
    #[must_use]
    pub fn compute_time_earnings(hours: &ShiftHoursBreakdown) -> Decimal {
        let reg = hours.regular_hours * hours.base_hourly_rate;
        let ot = hours.overtime_hours * (hours.base_hourly_rate * dec!(1.4));
        let hol = hours.holiday_hours * (hours.base_hourly_rate * dec!(2.0));
        let night = hours.night_shift_hours * (hours.base_hourly_rate * dec!(1.35));

        reg + ot + hol + night
    }

    /// Computes Piece-Rate Production Incentive:
    /// W_piece = sum( Q_accepted * r_unit - Q_scrap * p_penalty )
    #[must_use]
    pub fn compute_piece_earnings(job_cards: &[JobCardPieceworkLog]) -> Decimal {
        job_cards
            .iter()
            .map(|j| {
                let earned = j.qty_accepted * j.rate_per_unit;
                let penalty = j.qty_scrap * j.scrap_penalty_rate;
                (earned - penalty).max(Decimal::ZERO)
            })
            .sum()
    }

    /// Computes QA-approved milestone bonuses.
    #[must_use]
    pub fn compute_milestone_bonuses(milestones: &[MilestoneBonusLog]) -> Decimal {
        milestones
            .iter()
            .filter(|m| m.qa_approved)
            .map(|m| m.bonus_amount)
            .sum()
    }

    /// Calculates total gross payable applying the selected hybrid compensation model.
    #[must_use]
    pub fn calculate_worker_payout(
        worker_id: &str,
        cost_center: &str,
        model: CompensationModelType,
        hours: Option<&ShiftHoursBreakdown>,
        job_cards: &[JobCardPieceworkLog],
        milestones: &[MilestoneBonusLog],
        guaranteed_minimum: Decimal,
    ) -> DynamicCompensationResult {
        let time_earned = hours.map_or(Decimal::ZERO, Self::compute_time_earnings);
        let piece_earned = Self::compute_piece_earnings(job_cards);
        let milestone_earned = Self::compute_milestone_bonuses(milestones);

        let gross_payable = match model {
            CompensationModelType::TieredHourly => time_earned,
            CompensationModelType::PurePieceRate => piece_earned,
            CompensationModelType::ProjectMilestone => time_earned + milestone_earned,
            CompensationModelType::HybridGuaranteedMinimum => {
                let variable = piece_earned + milestone_earned;
                guaranteed_minimum.max(variable)
            }
        };

        DynamicCompensationResult {
            worker_id: worker_id.into(),
            model_used: model,
            time_based_earned: time_earned,
            piece_rate_earned: piece_earned,
            milestone_bonus_earned: milestone_earned,
            guaranteed_minimum,
            gross_payable,
            cost_center: cost_center.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiered_time_earnings() {
        let hours = ShiftHoursBreakdown {
            regular_hours: dec!(40),
            overtime_hours: dec!(10),
            holiday_hours: dec!(5),
            night_shift_hours: dec!(8),
            base_hourly_rate: dec!(20),
        };

        // 40*20=800, 10*(20*1.4)=280, 5*(20*2)=200, 8*(20*1.35)=216 -> Total: 1496.00
        let total = DynamicCompensationEngine::compute_time_earnings(&hours);
        assert_eq!(total, dec!(1496.00));
    }

    #[test]
    fn test_piece_rate_with_scrap_penalty() {
        let logs = vec![
            JobCardPieceworkLog {
                job_card_id: "JC-01".into(),
                operation: "CNC Milling".into(),
                qty_accepted: dec!(100),
                rate_per_unit: dec!(5.00),
                qty_scrap: dec!(4),
                scrap_penalty_rate: dec!(10.00),
            },
        ];

        // 100*5 = 500, penalty = 4*10 = 40 -> 460.00
        let piece = DynamicCompensationEngine::compute_piece_earnings(&logs);
        assert_eq!(piece, dec!(460.00));
    }

    #[test]
    fn test_hybrid_floor_guarantee() {
        let logs = vec![
            JobCardPieceworkLog {
                job_card_id: "JC-02".into(),
                operation: "Assembly".into(),
                qty_accepted: dec!(30),
                rate_per_unit: dec!(10.00),
                qty_scrap: dec!(0),
                scrap_penalty_rate: dec!(0),
            },
        ];

        // Earned $300, Guaranteed minimum $500 -> Payout is $500
        let payout = DynamicCompensationEngine::calculate_worker_payout(
            "WRK-100",
            "Plant Floor 1",
            CompensationModelType::HybridGuaranteedMinimum,
            None,
            &logs,
            &[],
            dec!(500.00),
        );

        assert_eq!(payout.gross_payable, dec!(500.00));
        assert_eq!(payout.piece_rate_earned, dec!(300.00));
    }
}
