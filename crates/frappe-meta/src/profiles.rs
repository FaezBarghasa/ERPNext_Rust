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
        ]
    }

    /// Retrieves a specific profile by ID.
    #[must_use]
    pub fn get_profile(id: &str) -> Option<VerticalProfile> {
        Self::list_profiles().into_iter().find(|p| p.profile_id.as_str() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_registry_listing() {
        let profiles = ProfileRegistry::list_profiles();
        assert_eq!(profiles.len(), 4);

        let restaurant = ProfileRegistry::get_profile("restaurant").unwrap();
        assert_eq!(restaurant.title, "The Rust Restaurant & Hospitality OS");
        assert!(restaurant.initial_doctypes.contains(&"KitchenOrderTicket".into()));

        let clinic = ProfileRegistry::get_profile("clinic").unwrap();
        assert!(clinic.initial_doctypes.contains(&"HipaaAuditLog".into()));
    }
}
