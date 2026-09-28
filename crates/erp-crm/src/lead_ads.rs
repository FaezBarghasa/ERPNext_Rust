use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Meta (Facebook / Instagram) Lead Ads Webhook field pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaFieldData {
    /// Field name / key (e.g. "full_name", "email", "phone_number").
    pub name: String,
    /// Submitted value array.
    pub values: Vec<String>,
}

/// Inbound Meta Lead Ads Webhook Payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaLeadAdPayload {
    /// Leadgen ID.
    pub leadgen_id: String,
    /// Page ID.
    pub page_id: String,
    /// Form ID.
    pub form_id: String,
    /// Ad ID.
    pub ad_id: Option<String>,
    /// Campaign ID.
    pub campaign_id: Option<String>,
    /// Created timestamp.
    pub created_time: i64,
    /// Field data values.
    pub field_data: Vec<MetaFieldData>,
}

/// UTM Attribution tracking parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UtmAttribution {
    /// utm_source (e.g. "meta", "google", "newsletter").
    pub utm_source: Option<String>,
    /// utm_medium (e.g. "cpc", "social", "email").
    pub utm_medium: Option<String>,
    /// utm_campaign (e.g. "spring_sale_2026").
    pub utm_campaign: Option<String>,
    /// utm_term.
    pub utm_term: Option<String>,
    /// utm_content.
    pub utm_content: Option<String>,
}

/// Ingested CRM Lead converted from Webhook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestedLead {
    /// Lead name.
    pub lead_name: String,
    /// Email address.
    pub email: Option<String>,
    /// Mobile / Phone number.
    pub phone: Option<String>,
    /// Source channel.
    pub source: String,
    /// Custom question-response answers.
    pub custom_fields: HashMap<String, String>,
    /// UTM tracking details.
    pub utm: UtmAttribution,
    /// Ingestion timestamp.
    pub received_at: DateTime<Utc>,
}

/// Meta Lead Ads Ingestion Webhook parser.
pub struct MetaLeadIngestor;

impl MetaLeadIngestor {
    /// Ingests and maps a Meta Lead Ads payload into a standard CRM IngestedLead.
    pub fn parse_webhook(payload: &MetaLeadAdPayload) -> IngestedLead {
        let mut lead_name = String::new();
        let mut email = None;
        let mut phone = None;
        let mut custom_fields = HashMap::new();

        for field in &payload.field_data {
            let val = field.values.first().cloned().unwrap_or_default();
            match field.name.as_str() {
                "full_name" | "name" | "first_name" => {
                    if lead_name.is_empty() {
                        lead_name = val;
                    }
                }
                "email" => email = Some(val),
                "phone_number" | "phone" => phone = Some(val),
                _ => {
                    custom_fields.insert(field.name.clone(), val);
                }
            }
        }

        if lead_name.is_empty() {
            lead_name = format!("Meta Lead {}", payload.leadgen_id);
        }

        let utm = UtmAttribution {
            utm_source: Some("meta".into()),
            utm_medium: Some("cpc".into()),
            utm_campaign: payload.campaign_id.clone(),
            utm_term: None,
            utm_content: payload.ad_id.clone(),
        };

        IngestedLead {
            lead_name,
            email,
            phone,
            source: "Meta Lead Ads".into(),
            custom_fields,
            utm,
            received_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_webhook_ingestion() {
        let payload = MetaLeadAdPayload {
            leadgen_id: "LG-12345".into(),
            page_id: "PAGE-99".into(),
            form_id: "FORM-88".into(),
            ad_id: Some("AD-77".into()),
            campaign_id: Some("CAMP-66".into()),
            created_time: 1774780000,
            field_data: vec![
                MetaFieldData {
                    name: "full_name".into(),
                    values: vec!["Jane Doe".into()],
                },
                MetaFieldData {
                    name: "email".into(),
                    values: vec!["jane@acme.com".into()],
                },
                MetaFieldData {
                    name: "phone_number".into(),
                    values: vec!["+15551234567".into()],
                },
                MetaFieldData {
                    name: "company_size".into(),
                    values: vec!["50-100".into()],
                },
            ],
        };

        let lead = MetaLeadIngestor::parse_webhook(&payload);
        assert_eq!(lead.lead_name, "Jane Doe");
        assert_eq!(lead.email, Some("jane@acme.com".into()));
        assert_eq!(lead.phone, Some("+15551234567".into()));
        assert_eq!(
            lead.custom_fields.get("company_size"),
            Some(&"50-100".to_string())
        );
        assert_eq!(lead.utm.utm_source, Some("meta".into()));
        assert_eq!(lead.utm.utm_campaign, Some("CAMP-66".into()));
    }
}
