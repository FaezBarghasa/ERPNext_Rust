//! Enterprise Operating System - Software, SaaS & Professional Services Engine.
//!
//! Provides comprehensive institutional-grade SaaS recurring billing, ASC 606 / IFRS 15
//! multi-element revenue recognition with zero-float drift, Professional Services Automation (PSA)
//! utilization tracking, and support SLA service credit ledgers.

pub mod errors;
pub mod psa;
pub mod revenue_recognition;
pub mod sla_ledger;
pub mod subscription;

pub use errors::SoftwareBillingError;
pub use psa::{
    EmployeeUtilizationScorecard, MilestoneInvoiceRequest, ProjectPsaSummary, PsaEngine,
    PsaInvoice, PsaInvoiceType, TimeLog,
};
pub use revenue_recognition::{
    AllocatedObligation, AmortizationScheduleEntry, Asc606Engine, CustomerContract,
    PerformanceObligation, RecognitionTiming,
};
pub use sla_ledger::{
    CreditLedgerEntry, CreditTransactionType, CustomerCreditLedger, SlaBreachPenalty, SlaPolicy,
    calculate_sla_penalty,
};
pub use subscription::{
    BillingInterval, InvoiceLineItem, MeteredUsageEvent, PlanPricingModel, Subscription,
    SubscriptionInvoice, SubscriptionPlan, SubscriptionStatus, UsageCollector, VolumeTier,
    calculate_graduated_tier_price, calculate_volume_tier_price, generate_subscription_invoice,
};
