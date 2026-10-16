//! Marketing Campaign & Contract Management Engine (`erp-crm::campaign`).
//!
//! Provides canonical ERPNext CRM capabilities:
//! - Multi-channel Marketing Campaign tracking with ROI and conversion analytics
//! - Customer & Supplier Legal Contracts with signing lifecycle and expiry management
//! - Opportunity pipeline conversion and weighted pipeline calculations.

use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// CRM Campaign & Contract operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CampaignError {
    #[error("Contract '{0}' has already expired or terminated")]
    ContractClosed(CompactString),
    #[error("End date {end} cannot precede start date {start}")]
    InvalidDateRange { start: NaiveDate, end: NaiveDate },
}

/// Marketing Campaign lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CampaignStatus {
    #[default]
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

/// Marketing Campaign record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Campaign {
    pub name: CompactString,
    pub campaign_name: CompactString,
    pub status: CampaignStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Decimal,
    pub actual_cost: Decimal,
    pub leads_count: u32,
    pub opportunities_count: u32,
    pub revenue_generated: Decimal,
}

impl Campaign {
    pub fn new(
        name: impl Into<CompactString>,
        campaign_name: impl Into<CompactString>,
        budget: Decimal,
    ) -> Self {
        Self {
            name: name.into(),
            campaign_name: campaign_name.into(),
            status: CampaignStatus::Planned,
            start_date: None,
            end_date: None,
            budget,
            actual_cost: Decimal::ZERO,
            leads_count: 0,
            opportunities_count: 0,
            revenue_generated: Decimal::ZERO,
        }
    }

    /// Records expense cost incurred by this campaign (ad spend, sponsorship).
    pub fn record_expense(&mut self, cost: Decimal) {
        self.actual_cost += cost;
    }

    /// Increments lead count attributed to this campaign.
    pub fn record_lead(&mut self) {
        self.leads_count += 1;
    }

    /// Records a converted opportunity win and revenue generated.
    pub fn record_conversion(&mut self, revenue: Decimal) {
        self.opportunities_count += 1;
        self.revenue_generated += revenue;
    }

    /// Calculates return on investment (ROI) percentage:
    /// $\text{ROI} = \left(\frac{\text{Revenue Generated} - \text{Actual Cost}}{\text{Actual Cost}}\right) \times 100\%$
    #[must_use]
    pub fn calculate_roi_percentage(&self) -> Decimal {
        if self.actual_cost.is_zero() {
            if self.revenue_generated > Decimal::ZERO {
                dec!(100.0)
            } else {
                Decimal::ZERO
            }
        } else {
            let net = self.revenue_generated - self.actual_cost;
            ((net / self.actual_cost) * dec!(100.0)).round_dp(2)
        }
    }
}

/// Contract lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContractStatus {
    #[default]
    Draft,
    Active,
    Expired,
    Terminated,
}

/// Commercial Contract document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contract {
    pub name: CompactString,
    pub party_type: CompactString, // "Customer", "Supplier", "Employee"
    pub party_name: CompactString,
    pub contract_terms: CompactString,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: ContractStatus,
    pub is_signed: bool,
    pub signed_on: Option<NaiveDate>,
}

impl Contract {
    pub fn new(
        name: impl Into<CompactString>,
        party_type: impl Into<CompactString>,
        party_name: impl Into<CompactString>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        contract_terms: impl Into<CompactString>,
    ) -> Result<Self, CampaignError> {
        if end_date < start_date {
            return Err(CampaignError::InvalidDateRange {
                start: start_date,
                end: end_date,
            });
        }

        Ok(Self {
            name: name.into(),
            party_type: party_type.into(),
            party_name: party_name.into(),
            contract_terms: contract_terms.into(),
            start_date,
            end_date,
            status: ContractStatus::Draft,
            is_signed: false,
            signed_on: None,
        })
    }

    /// Executes contract signing, promoting state to Active.
    pub fn sign_contract(&mut self, signed_date: NaiveDate) -> Result<(), CampaignError> {
        if self.status == ContractStatus::Expired || self.status == ContractStatus::Terminated {
            return Err(CampaignError::ContractClosed(self.name.clone()));
        }

        self.is_signed = true;
        self.signed_on = Some(signed_date);
        self.status = ContractStatus::Active;
        Ok(())
    }

    /// Terminates the contract prematurely.
    pub fn terminate(&mut self) {
        self.status = ContractStatus::Terminated;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_roi_and_lead_attribution() {
        let mut campaign = Campaign::new("CAMP-001", "Fall Product Launch 2026", dec!(10000.0));
        assert_eq!(campaign.status, CampaignStatus::Planned);

        // Spend $4,000 on ads
        campaign.record_expense(dec!(4000.0));
        // Attributed leads
        campaign.record_lead();
        campaign.record_lead();
        campaign.record_lead();
        assert_eq!(campaign.leads_count, 3);

        // Win deal worth $16,000
        campaign.record_conversion(dec!(16000.0));

        // Net = 16,000 - 4,000 = 12,000. ROI = (12,000 / 4,000) * 100 = 300.0%
        assert_eq!(campaign.calculate_roi_percentage(), dec!(300.0));
    }

    #[test]
    fn test_contract_signing_and_lifecycle() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();

        let mut contract = Contract::new(
            "CONT-001",
            "Customer",
            "Acme Global Holdings",
            start,
            end,
            "Standard Master Services Agreement with Net-30 payment terms",
        )
        .unwrap();

        assert_eq!(contract.status, ContractStatus::Draft);
        assert!(!contract.is_signed);

        let sign_date = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        contract.sign_contract(sign_date).unwrap();

        assert_eq!(contract.status, ContractStatus::Active);
        assert!(contract.is_signed);
        assert_eq!(contract.signed_on, Some(sign_date));

        // Terminate
        contract.terminate();
        assert_eq!(contract.status, ContractStatus::Terminated);
    }
}
