use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Opportunity / Deal Pipeline Stage with standard win probability weighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealStage {
    /// Initial discovery / qualification (10% probability).
    Qualification,
    /// Requirement analysis & scope definition (25% probability).
    NeedsAnalysis,
    /// Proposal / Quote submitted (50% probability).
    Proposal,
    /// Legal / Price negotiation (80% probability).
    Negotiation,
    /// Deal won and committed (100% probability).
    ClosedWon,
    /// Deal lost (0% probability).
    ClosedLost,
}

impl DealStage {
    /// Default probability weight associated with stage.
    pub fn default_probability(&self) -> Decimal {
        match self {
            Self::Qualification => dec!(0.10),
            Self::NeedsAnalysis => dec!(0.25),
            Self::Proposal => dec!(0.50),
            Self::Negotiation => dec!(0.80),
            Self::ClosedWon => dec!(1.00),
            Self::ClosedLost => dec!(0.00),
        }
    }
}

/// Opportunity deal record for pipeline forecasting and SLA tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpportunityDeal {
    /// Deal ID.
    pub name: String,
    /// Title.
    pub title: String,
    /// Total expected deal value in base currency.
    pub deal_amount: Decimal,
    /// Current pipeline stage.
    pub stage: DealStage,
    /// Expected closing date.
    pub expected_close_date: NaiveDate,
    /// Custom probability override (if set), otherwise defaults to stage probability.
    pub custom_probability: Option<Decimal>,
    /// Date deal was opened.
    pub created_date: NaiveDate,
    /// Date of last activity / stage update.
    pub last_activity_date: NaiveDate,
    /// First response recorded date.
    pub first_response_date: Option<NaiveDate>,
}

impl OpportunityDeal {
    /// Resolves active probability percentage for this deal.
    pub fn probability(&self) -> Decimal {
        self.custom_probability
            .unwrap_or_else(|| self.stage.default_probability())
    }

    /// Computes weighted revenue forecast amount: `deal_amount * probability`.
    pub fn weighted_amount(&self) -> Decimal {
        self.deal_amount * self.probability()
    }
}

/// Aggregated pipeline forecast summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineForecastSummary {
    /// Total nominal unweighted pipeline value.
    pub total_nominal_value: Decimal,
    /// Total probability-weighted pipeline forecast.
    pub total_weighted_forecast: Decimal,
    /// Total count of open active deals.
    pub open_deal_count: usize,
    /// Deals broken down by stage.
    pub stage_breakdown: Vec<(DealStage, usize, Decimal, Decimal)>,
}

/// SLA rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DealSlaConfig {
    /// Maximum allowable days before first response is made.
    pub max_first_response_days: i64,
    /// Maximum allowable days a deal can remain idle in one stage without activity.
    pub max_stage_idle_days: i64,
}

/// SLA violation diagnosis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlaBreachReason {
    /// First response SLA exceeded.
    FirstResponseBreached { days_elapsed: i64, limit: i64 },
    /// Stage idle duration SLA exceeded.
    StageIdleBreached { days_idle: i64, limit: i64 },
}

/// Deal SLA and Weighted Forecasting Engine.
pub struct PipelineForecastEngine;

impl PipelineForecastEngine {
    /// Computes comprehensive weighted forecast across all active pipeline deals.
    pub fn compute_forecast(deals: &[OpportunityDeal]) -> PipelineForecastSummary {
        let mut total_nominal = Decimal::ZERO;
        let mut total_weighted = Decimal::ZERO;
        let mut count = 0;

        let stages = [
            DealStage::Qualification,
            DealStage::NeedsAnalysis,
            DealStage::Proposal,
            DealStage::Negotiation,
            DealStage::ClosedWon,
            DealStage::ClosedLost,
        ];

        let mut breakdown = Vec::new();

        for stage in stages {
            let stage_deals: Vec<&OpportunityDeal> =
                deals.iter().filter(|d| d.stage == stage).collect();
            let stage_count = stage_deals.len();
            let nominal: Decimal = stage_deals.iter().map(|d| d.deal_amount).sum();
            let weighted: Decimal = stage_deals.iter().map(|d| d.weighted_amount()).sum();

            if stage != DealStage::ClosedLost && stage != DealStage::ClosedWon {
                total_nominal += nominal;
                total_weighted += weighted;
                count += stage_count;
            }

            breakdown.push((stage, stage_count, nominal, weighted));
        }

        PipelineForecastSummary {
            total_nominal_value: total_nominal,
            total_weighted_forecast: total_weighted,
            open_deal_count: count,
            stage_breakdown: breakdown,
        }
    }

    /// Evaluates SLA compliance for a deal as of a given target date.
    pub fn check_sla(
        deal: &OpportunityDeal,
        as_of: NaiveDate,
        config: &DealSlaConfig,
    ) -> Vec<SlaBreachReason> {
        let mut breaches = Vec::new();

        // 1. Check First Response SLA
        match deal.first_response_date {
            Some(resp_date) => {
                let days = (resp_date - deal.created_date).num_days();
                if days > config.max_first_response_days {
                    breaches.push(SlaBreachReason::FirstResponseBreached {
                        days_elapsed: days,
                        limit: config.max_first_response_days,
                    });
                }
            }
            None => {
                let days = (as_of - deal.created_date).num_days();
                if days > config.max_first_response_days {
                    breaches.push(SlaBreachReason::FirstResponseBreached {
                        days_elapsed: days,
                        limit: config.max_first_response_days,
                    });
                }
            }
        }

        // 2. Check Stage Idle SLA
        let days_idle = (as_of - deal.last_activity_date).num_days();
        if days_idle > config.max_stage_idle_days {
            breaches.push(SlaBreachReason::StageIdleBreached {
                days_idle,
                limit: config.max_stage_idle_days,
            });
        }

        breaches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_forecast_and_sla_breach() {
        let date_created = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let date_active = NaiveDate::from_ymd_opt(2026, 9, 10).unwrap();
        let as_of = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();

        let deals = vec![
            OpportunityDeal {
                name: "OPP-01".into(),
                title: "Cloud Migration".into(),
                deal_amount: dec!(100000.0),
                stage: DealStage::Proposal, // 50% -> 50,000
                expected_close_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                custom_probability: None,
                created_date: date_created,
                last_activity_date: date_active,
                first_response_date: Some(NaiveDate::from_ymd_opt(2026, 9, 2).unwrap()),
            },
            OpportunityDeal {
                name: "OPP-02".into(),
                title: "Security Audit".into(),
                deal_amount: dec!(50000.0),
                stage: DealStage::Negotiation, // 80% -> 40,000
                expected_close_date: NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
                custom_probability: None,
                created_date: date_created,
                last_activity_date: date_active,
                first_response_date: None, // Breached
            },
        ];

        let summary = PipelineForecastEngine::compute_forecast(&deals);
        assert_eq!(summary.total_nominal_value, dec!(150000.0));
        assert_eq!(summary.total_weighted_forecast, dec!(90000.0));
        assert_eq!(summary.open_deal_count, 2);

        let sla_cfg = DealSlaConfig {
            max_first_response_days: 2,
            max_stage_idle_days: 14,
        };

        // OPP-01: First response took 1 day (OK), idle since Sept 10 to Sept 28 is 18 days (> 14 -> Breached)
        let breaches_01 = PipelineForecastEngine::check_sla(&deals[0], as_of, &sla_cfg);
        assert_eq!(breaches_01.len(), 1);
        assert!(matches!(
            breaches_01[0],
            SlaBreachReason::StageIdleBreached { .. }
        ));

        // OPP-02: No first response and 27 days elapsed (Breached) + idle 18 days (Breached)
        let breaches_02 = PipelineForecastEngine::check_sla(&deals[1], as_of, &sla_cfg);
        assert_eq!(breaches_02.len(), 2);
    }
}
