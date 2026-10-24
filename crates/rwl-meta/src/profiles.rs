//! Curated "Business-in-a-Box" Vertical Profiles (`frappe-meta::profiles`).
//!
//! Provides pre-configured, production-ready enterprise templates deployed in <180 seconds:
//! - The Rust Restaurant (Floor plans, KDS, Recipe BOMs, Tip payroll, POS)
//! - The Rust Clinic (EMR, Practitioner rosters, Insurance billing, HIPAA audit)
//! - The Rust E-Commerce Store (Multi-warehouse stock, Carrier integrations, Automated VAT/GST)
//! - The Professional Agency (Timesheets, Milestone billing, Retainers, AIA G702, CRM)

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Pre-configured vertical profile definition.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VerticalProfile {
    pub profile_id: CompactString,
    pub title: CompactString,
    pub description: CompactString,
    pub target_industry: CompactString,
    pub initial_doctypes: Vec<CompactString>,
    pub initial_roles: Vec<CompactString>,
    pub default_coa_template: CompactString,
    pub default_tax_template: CompactString,
}

pub struct ProfileRegistry;

impl ProfileRegistry {
    /// Returns all available out-of-the-box vertical profiles.
    #[must_use]
    pub fn list_profiles() -> Vec<VerticalProfile> {
        vec![
            VerticalProfile {
                profile_id: "restaurant".into(),
                title: "The Rust Restaurant & Hospitality OS".into(),
                description: "Floor plans, Kitchen Display System (KDS), Recipe BOMs, Tip Distribution Payroll, POS".into(),
                target_industry: "Food & Beverage".into(),
                initial_doctypes: vec![
                    "DiningTable".into(),
                    "KitchenOrderTicket".into(),
                    "RecipeBOM".into(),
                    "TipDistributionLog".into(),
                    "PosInvoice".into(),
                ],
                initial_roles: vec!["Server".into(), "Chef".into(), "RestaurantManager".into()],
                default_coa_template: "Hospitality Standard CoA".into(),
                default_tax_template: "Standard F&B Tax (10% Sales + 5% Hospitality)".into(),
            },
            VerticalProfile {
                profile_id: "clinic".into(),
                title: "The Rust Medical Clinic & Health OS".into(),
                description: "Electronic Medical Records (EMR), Practitioner Rosters, Insurance Billing, HIPAA Logs".into(),
                target_industry: "Healthcare".into(),
                initial_doctypes: vec![
                    "PatientRecord".into(),
                    "DoctorAppointment".into(),
                    "Prescription".into(),
                    "InsuranceClaim".into(),
                    "HipaaAuditLog".into(),
                ],
                initial_roles: vec!["Physician".into(), "Nurse".into(), "ClinicAdministrator".into()],
                default_coa_template: "Healthcare Standard CoA".into(),
                default_tax_template: "Medical Tax Exempt / Standard VAT".into(),
            },
            VerticalProfile {
                profile_id: "ecommerce".into(),
                title: "The Rust E-Commerce & Retail OS".into(),
                description: "Multi-Warehouse Inventory, Courier Dispatch (DHL/FedEx), Automated VAT/GST, Storefront".into(),
                target_industry: "Digital Commerce".into(),
                initial_doctypes: vec![
                    "ProductCatalog".into(),
                    "WarehouseBin".into(),
                    "CourierShipment".into(),
                    "OnlineOrder".into(),
                    "CustomerCart".into(),
                ],
                initial_roles: vec!["StorefrontCustomer".into(), "WarehousePicker".into(), "StoreManager".into()],
                default_coa_template: "Retail E-Commerce CoA".into(),
                default_tax_template: "Automated Destination VAT/GST".into(),
            },
            VerticalProfile {
                profile_id: "agency".into(),
                title: "The Professional Agency & Services OS".into(),
                description: "Timesheets, Milestone Billing, Retainers, AIA G702 Invoicing, Buying Center CRM".into(),
                target_industry: "Professional Services".into(),
                initial_doctypes: vec![
                    "AgencyProject".into(),
                    "TimesheetEntry".into(),
                    "MilestoneInvoice".into(),
                    "RetainerContract".into(),
                    "BuyingCenterContact".into(),
                ],
                initial_roles: vec!["AccountExecutive".into(), "Consultant".into(), "FinanceDirector".into()],
                default_coa_template: "Professional Services Standard CoA".into(),
                default_tax_template: "Service Tax 18% / Standard VAT".into(),
            },
            VerticalProfile {
                profile_id: "svod-streaming".into(),
                title: "SVoD Video Streaming Platform (Netflix / 30nama Class)".into(),
                description: "HLS Transcoder, 1536-dim Subtitle Vector Search, Watch Party WebSockets, Studio Royalties".into(),
                target_industry: "Media & Entertainment".into(),
                initial_doctypes: vec![
                    "VideoAsset".into(),
                    "MediaTranscript".into(),
                    "SubscriptionPlan".into(),
                    "StreamSession".into(),
                    "RoyaltyAccrual".into(),
                ],
                initial_roles: vec!["Subscriber".into(), "ContentStudioPartner".into(), "PlatformAdmin".into()],
                default_coa_template: "Media Entertainment Standard CoA".into(),
                default_tax_template: "Digital Services Tax / Standard VAT".into(),
            },
            VerticalProfile {
                profile_id: "lms-academy".into(),
                title: "Digital Learning & LMS Academy (Coursera / Skillshare Class)".into(),
                description: "Course Graph Trees, ASC 606 Tuition Amortization, Typst PDF/A Graduation Diplomas, Quizzes".into(),
                target_industry: "Education & EdTech".into(),
                initial_doctypes: vec![
                    "Course".into(),
                    "LearningModule".into(),
                    "LessonAssessment".into(),
                    "StudentEnrollment".into(),
                    "GraduationCertificate".into(),
                ],
                initial_roles: vec!["Student".into(), "Instructor".into(), "AcademicDirector".into()],
                default_coa_template: "Higher Education Standard CoA".into(),
                default_tax_template: "Education Tax Exempt / Standard VAT".into(),
            },
            VerticalProfile {
                profile_id: "digital-goods".into(),
                title: "Digital Products & Creator Hub (Gumroad / LemonSqueezy Class)".into(),
                description: "Encrypted Signed URLs, Node-Locked Licenses, Split Payouts, Global EU VAT MOSS".into(),
                target_industry: "Creator Economy & Software".into(),
                initial_doctypes: vec![
                    "DigitalProduct".into(),
                    "LicenseKey".into(),
                    "DownloadGrant".into(),
                    "CreatorPayoutSplit".into(),
                    "AffiliateReferral".into(),
                ],
                initial_roles: vec!["Creator".into(), "Licensee".into(), "PlatformAuditor".into()],
                default_coa_template: "Software & Creator Economy CoA".into(),
                default_tax_template: "EU VAT MOSS / US State Sales Tax".into(),
            },
            VerticalProfile {
                profile_id: "b2b-industrial".into(),
                title: "Industrial B2B & Wholesale Matrix (Grainger / Misumi Class)".into(),
                description: "WebGL CAD 3D Exploded Viewer, Tiered Price Matrix, Corporate Net Terms, ZUGFeRD E-Invoice".into(),
                target_industry: "Industrial Manufacturing & Wholesale".into(),
                initial_doctypes: vec![
                    "B2bItem".into(),
                    "WholesalePriceMatrix".into(),
                    "TradeCreditAccount".into(),
                    "PurchaseRequisition".into(),
                    "ZugferdInvoice".into(),
                ],
                initial_roles: vec!["ProcurementOfficer".into(), "WholesaleCustomer".into(), "CreditRiskManager".into()],
                default_coa_template: "Industrial Manufacturing & Wholesale CoA".into(),
                default_tax_template: "Standard Corporate VAT / Reverse Charge".into(),
            },
            VerticalProfile {
                profile_id: "b2c-retail".into(),
                title: "Consumer B2C Omnichannel Flagship (Shopify Killer / ASOS Class)".into(),
                description: "60 FPS Viewport Grid, Real-Time Flash Stock Feed, Slide-Over Drawer, FIFO Stock Reservation".into(),
                target_industry: "Retail & Consumer Goods".into(),
                initial_doctypes: vec![
                    "RetailProduct".into(),
                    "FlashSaleEvent".into(),
                    "CustomerCart".into(),
                    "PickWave".into(),
                    "LoyaltyAccount".into(),
                ],
                initial_roles: vec!["RetailShopper".into(), "FulfillmentSpecialist".into(), "Merchandiser".into()],
                default_coa_template: "Omnichannel Retail & CPG CoA".into(),
                default_tax_template: "Destination-Based Sales Tax & VAT".into(),
            },
            VerticalProfile {
                profile_id: "trading-exchange".into(),
                title: "Financial Trading & Brokerage Hub (Robinhood / TradingView Class)".into(),
                description: "WebGL Canvas Candlesticks, Streaming Level-2 Depth Book, Sanctions KYC, Multi-Currency Wallets".into(),
                target_industry: "Fintech & Capital Markets".into(),
                initial_doctypes: vec![
                    "TradingPair".into(),
                    "ExchangeOrder".into(),
                    "WalletLedger".into(),
                    "KycProfile".into(),
                    "MerkleSettlementBlock".into(),
                ],
                initial_roles: vec!["Trader".into(), "BrokerDealer".into(), "ComplianceOfficer".into()],
                default_coa_template: "Brokerage & Treasury Multi-Currency CoA".into(),
                default_tax_template: "Financial Transaction Tax Exempt / Capital Gains".into(),
            },
        ]
    }

    /// Retrieves a specific profile by ID.
    #[must_use]
    pub fn get_profile(id: &str) -> Option<VerticalProfile> {
        Self::list_profiles()
            .into_iter()
            .find(|p| p.profile_id.as_str() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_registry_listing() {
        let profiles = ProfileRegistry::list_profiles();
        assert_eq!(profiles.len(), 10);

        let restaurant = ProfileRegistry::get_profile("restaurant").unwrap();
        assert_eq!(restaurant.title, "The Rust Restaurant & Hospitality OS");
        assert!(
            restaurant
                .initial_doctypes
                .contains(&"KitchenOrderTicket".into())
        );

        let clinic = ProfileRegistry::get_profile("clinic").unwrap();
        assert!(clinic.initial_doctypes.contains(&"HipaaAuditLog".into()));

        // Verify the 6 prebuilt template archetypes
        let svod = ProfileRegistry::get_profile("svod-streaming").unwrap();
        assert!(svod.initial_doctypes.contains(&"VideoAsset".into()));

        let lms = ProfileRegistry::get_profile("lms-academy").unwrap();
        assert!(
            lms.initial_doctypes
                .contains(&"GraduationCertificate".into())
        );

        let digital = ProfileRegistry::get_profile("digital-goods").unwrap();
        assert!(digital.initial_doctypes.contains(&"LicenseKey".into()));

        let b2b = ProfileRegistry::get_profile("b2b-industrial").unwrap();
        assert!(b2b.initial_doctypes.contains(&"ZugferdInvoice".into()));

        let b2c = ProfileRegistry::get_profile("b2c-retail").unwrap();
        assert!(b2c.initial_doctypes.contains(&"FlashSaleEvent".into()));

        let trading = ProfileRegistry::get_profile("trading-exchange").unwrap();
        assert!(trading.initial_doctypes.contains(&"ExchangeOrder".into()));
    }
}
