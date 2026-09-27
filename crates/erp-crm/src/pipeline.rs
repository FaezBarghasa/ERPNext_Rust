use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// CRM domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CrmError {
    /// Invalid state machine transition.
    #[error("Invalid CRM stage transition from {from:?} to {to:?}")]
    InvalidTransition { from: LeadStatus, to: LeadStatus },
    /// Quotation has expired.
    #[error("Quotation '{name}' has expired on {valid_till}")]
    QuotationExpired { name: String, valid_till: NaiveDate },
    /// Quotation is not approved.
    #[error("Quotation '{name}' must be in Submitted state to convert to Sales Order")]
    QuotationNotSubmitted { name: String },
}

/// Lead progression status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadStatus {
    Open,
    Replied,
    Opportunity,
    Converted,
    Lost,
}

impl LeadStatus {
    /// Validates state transition progression.
    pub fn can_transition_to(self, target: LeadStatus) -> bool {
        match (self, target) {
            (
                LeadStatus::Open,
                LeadStatus::Replied | LeadStatus::Opportunity | LeadStatus::Lost,
            ) => true,
            (LeadStatus::Replied, LeadStatus::Opportunity | LeadStatus::Lost) => true,
            (LeadStatus::Opportunity, LeadStatus::Converted | LeadStatus::Lost) => true,
            (LeadStatus::Lost, LeadStatus::Open) => true,
            _ => false,
        }
    }
}

/// CRM Lead record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lead {
    pub name: String,
    pub lead_name: String,
    pub email: String,
    pub status: LeadStatus,
}

impl Lead {
    /// Mutates lead state.
    pub fn transition_to(&mut self, next_status: LeadStatus) -> Result<(), CrmError> {
        if self.status.can_transition_to(next_status) {
            self.status = next_status;
            Ok(())
        } else {
            Err(CrmError::InvalidTransition {
                from: self.status,
                to: next_status,
            })
        }
    }
}

/// Item line in a sales quotation or order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuotationItem {
    pub item_code: String,
    pub qty: Decimal,
    pub rate: Decimal,
    pub amount: Decimal,
}

/// Quotation document status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotationStatus {
    Draft,
    Submitted,
    Ordered,
    Cancelled,
}

/// Commercial Quotation locking pricing and terms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Quotation {
    pub name: String,
    pub party_name: String,
    pub valid_till: NaiveDate,
    pub items: Vec<QuotationItem>,
    pub net_total: Decimal,
    pub status: QuotationStatus,
}

/// Confirmed Sales Order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SalesOrder {
    pub name: String,
    pub customer: String,
    pub quotation_ref: String,
    pub items: Vec<QuotationItem>,
    pub net_total: Decimal,
    pub posting_date: NaiveDate,
}

/// CRM Pipeline Coordinator (Milestone 3.9).
pub struct CrmPipeline;

impl CrmPipeline {
    /// Converts an approved Quotation into a confirmed SalesOrder, preserving locked pricing.
    pub fn convert_to_sales_order(
        quotation: &mut Quotation,
        as_of_date: NaiveDate,
        sales_order_id: &str,
    ) -> Result<SalesOrder, CrmError> {
        if quotation.status != QuotationStatus::Submitted {
            return Err(CrmError::QuotationNotSubmitted {
                name: quotation.name.clone(),
            });
        }

        if as_of_date > quotation.valid_till {
            return Err(CrmError::QuotationExpired {
                name: quotation.name.clone(),
                valid_till: quotation.valid_till,
            });
        }

        quotation.status = QuotationStatus::Ordered;

        Ok(SalesOrder {
            name: sales_order_id.to_string(),
            customer: quotation.party_name.clone(),
            quotation_ref: quotation.name.clone(),
            items: quotation.items.clone(),
            net_total: quotation.net_total,
            posting_date: as_of_date,
        })
    }
}
