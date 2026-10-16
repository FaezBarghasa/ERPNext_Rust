//! Upstream ERPNext CRM Golden Tests (`erp-crm`).
//!
//! Validates:
//! 1. Lead qualification lifecycle: Open -> Replied -> Opportunity -> Converted.
//! 2. Multi-channel Campaign marketing spend, lead attribution, and ROI calculation.
//! 3. Commercial Contract management, execution dates, and signing status.

use chrono::NaiveDate;
use erp_crm::{Campaign, Contract, ContractStatus, Lead, LeadStatus};
use rust_decimal_macros::dec;

#[test]
fn test_golden_lead_and_campaign_roi_conversion() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

    // 1. Setup Marketing Campaign
    let mut campaign = Campaign::new(
        "CAMP-2026-AUTUMN",
        "Autumn Industrial Automation Expo",
        dec!(25000.0), // $25k budget
    );
    campaign.start_date = Some(date);
    campaign.record_expense(dec!(15000.0)); // Incurred $15k

    // 2. Ingest Lead from Campaign
    let mut lead = Lead {
        name: "LEAD-2026-0042".into(),
        lead_name: "Sarah Chen".into(),
        email: "schen@quantumrobotics.io".into(),
        status: LeadStatus::Open,
    };
    campaign.record_lead();
    assert_eq!(campaign.leads_count, 1);

    // 3. Lead Qualification Workflow
    lead.transition_to(LeadStatus::Replied).unwrap();
    assert_eq!(lead.status, LeadStatus::Replied);

    lead.transition_to(LeadStatus::Opportunity).unwrap();
    assert_eq!(lead.status, LeadStatus::Opportunity);

    lead.transition_to(LeadStatus::Converted).unwrap();
    assert_eq!(lead.status, LeadStatus::Converted);

    // 4. Record Win into Campaign: $75,000 deal
    campaign.record_conversion(dec!(75000.0));
    assert_eq!(campaign.opportunities_count, 1);
    assert_eq!(campaign.revenue_generated, dec!(75000.0));

    // ROI: (75,000 - 15,000) / 15,000 = 60,000 / 15,000 = 400.0%
    assert_eq!(campaign.calculate_roi_percentage(), dec!(400.0));
}

#[test]
fn test_golden_contract_execution() {
    let start = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2027, 10, 31).unwrap();

    let mut contract = Contract::new(
        "CONT-2026-089",
        "Customer",
        "Quantum Robotics Inc",
        start,
        end,
        "Enterprise Multi-site Software License & 24/7 SLA Support",
    )
    .unwrap();

    assert_eq!(contract.status, ContractStatus::Draft);
    assert!(!contract.is_signed);

    // Sign contract on Oct 20
    let sign_date = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    contract.sign_contract(sign_date).unwrap();

    assert_eq!(contract.status, ContractStatus::Active);
    assert!(contract.is_signed);
    assert_eq!(contract.signed_on, Some(sign_date));
}
