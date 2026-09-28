use serde::{Deserialize, Serialize};

/// Contact person associated with a corporate Prospect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProspectContact {
    /// Contact full name.
    pub full_name: String,
    /// Email address.
    pub email: String,
    /// Mobile / Phone.
    pub phone: Option<String>,
    /// Job title / designation.
    pub designation: Option<String>,
    /// Primary contact flag.
    pub is_primary: bool,
}

/// Opportunity summary linked under a Prospect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProspectOpportunity {
    /// Opportunity document name.
    pub opportunity_id: String,
    /// Deal title.
    pub title: String,
    /// Expected deal value.
    pub expected_value: u64,
    /// Current stage.
    pub stage: String,
}

/// Prospect document representing an uncommitted organization with multiple stakeholders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prospect {
    /// Prospect identifier.
    pub name: String,
    /// Company / Organization name.
    pub company_name: String,
    /// Industry domain.
    pub industry: Option<String>,
    /// Annual revenue / budget tier.
    pub annual_revenue: Option<u64>,
    /// Associated contacts.
    pub contacts: Vec<ProspectContact>,
    /// Linked opportunities.
    pub opportunities: Vec<ProspectOpportunity>,
    /// Converted customer ID if already converted.
    pub customer_id: Option<String>,
}

/// Customer entity generated upon conversion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConvertedCustomer {
    /// Customer ID.
    pub customer_id: String,
    /// Customer company name.
    pub customer_name: String,
    /// Primary contact email.
    pub primary_email: Option<String>,
    /// Primary contact phone.
    pub primary_phone: Option<String>,
    /// Originating Prospect reference.
    pub prospect_ref: String,
}

impl Prospect {
    /// Converts a Prospect into a Customer, linking primary contact details.
    pub fn convert_to_customer(&mut self, new_customer_id: &str) -> ConvertedCustomer {
        self.customer_id = Some(new_customer_id.to_string());

        let primary_contact = self
            .contacts
            .iter()
            .find(|c| c.is_primary)
            .or_else(|| self.contacts.first());

        ConvertedCustomer {
            customer_id: new_customer_id.to_string(),
            customer_name: self.company_name.clone(),
            primary_email: primary_contact.map(|c| c.email.clone()),
            primary_phone: primary_contact.and_then(|c| c.phone.clone()),
            prospect_ref: self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prospect_to_customer_conversion() {
        let mut prospect = Prospect {
            name: "PROSP-2026-001".into(),
            company_name: "Apex Aerospace".into(),
            industry: Some("Aviation".into()),
            annual_revenue: Some(5_000_000),
            contacts: vec![
                ProspectContact {
                    full_name: "Alice VP".into(),
                    email: "alice@apex.com".into(),
                    phone: Some("+1-555-0100".into()),
                    designation: Some("VP Procurement".into()),
                    is_primary: true,
                },
                ProspectContact {
                    full_name: "Bob Engineer".into(),
                    email: "bob@apex.com".into(),
                    phone: None,
                    designation: Some("Chief Engineer".into()),
                    is_primary: false,
                },
            ],
            opportunities: vec![],
            customer_id: None,
        };

        let cust = prospect.convert_to_customer("CUST-APEX-01");
        assert_eq!(cust.customer_name, "Apex Aerospace");
        assert_eq!(cust.primary_email, Some("alice@apex.com".into()));
        assert_eq!(cust.primary_phone, Some("+1-555-0100".into()));
        assert_eq!(prospect.customer_id, Some("CUST-APEX-01".into()));
    }
}
