pub mod buying_center;
pub mod clm;
pub mod cpq;
pub mod pipeline;
pub mod revops;
pub mod scoring;

pub use buying_center::{
    BuyingCenterGraph, CorporateHierarchy, InfluenceEdge, Stakeholder, StakeholderRole,
};
pub use clm::{ClauseLibrary, ClauseVariant, ContractClause, ContractRedliner};
pub use cpq::{CpqSolver, OptionConstraint, PriceWaterfall};
pub use pipeline::{
    CrmError, CrmPipeline, Lead, LeadStatus, Quotation, QuotationItem, QuotationStatus, SalesOrder,
};
pub use revops::{PerformanceObligation, RevOpsEngine, RevenueContract, SatisfactionMethod};
pub use scoring::calculate_lead_score;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_crm_lead_qualification_workflow() {
        let mut lead = Lead {
            name: "LEAD-001".into(),
            lead_name: "John Doe".into(),
            email: "john@example.com".into(),
            status: LeadStatus::Open,
        };

        // Open -> Replied -> Opportunity -> Converted
        assert!(lead.transition_to(LeadStatus::Replied).is_ok());
        assert_eq!(lead.status, LeadStatus::Replied);

        assert!(lead.transition_to(LeadStatus::Opportunity).is_ok());
        assert_eq!(lead.status, LeadStatus::Opportunity);

        assert!(lead.transition_to(LeadStatus::Converted).is_ok());
        assert_eq!(lead.status, LeadStatus::Converted);

        // Converted cannot go to Replied
        assert!(matches!(
            lead.transition_to(LeadStatus::Replied),
            Err(CrmError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_quotation_conversion_and_price_locking() {
        let mut quote = Quotation {
            name: "QTN-2026-001".into(),
            party_name: "Acme Corp".into(),
            valid_till: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            items: vec![QuotationItem {
                item_code: "PRODUCT-A".into(),
                qty: dec!(5.0),
                rate: dec!(100.00),
                amount: dec!(500.00),
            }],
            net_total: dec!(500.00),
            status: QuotationStatus::Submitted,
        };

        let as_of = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let sales_order = CrmPipeline::convert_to_sales_order(&mut quote, as_of, "SO-2026-001")
            .expect("Conversion failed");

        assert_eq!(sales_order.customer, "Acme Corp");
        assert_eq!(sales_order.quotation_ref, "QTN-2026-001");
        assert_eq!(sales_order.net_total, dec!(500.00));
        assert_eq!(quote.status, QuotationStatus::Ordered);

        // Test expired quote conversion rejection
        let mut expired_quote = Quotation {
            name: "QTN-2026-EXP".into(),
            party_name: "Beta Corp".into(),
            valid_till: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            items: vec![],
            net_total: dec!(0.0),
            status: QuotationStatus::Submitted,
        };
        let late_date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let err = CrmPipeline::convert_to_sales_order(&mut expired_quote, late_date, "SO-EXP");
        assert!(matches!(err, Err(CrmError::QuotationExpired { .. })));
    }
}
