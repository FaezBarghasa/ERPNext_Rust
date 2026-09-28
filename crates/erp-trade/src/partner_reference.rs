use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Partner entity category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartnerType {
    /// Customer reference mapping.
    Customer,
    /// Supplier part number mapping.
    Supplier,
}

/// Cross-reference entry linking internal Item Code with Partner's external part numbering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartnerReference {
    /// Internal ERP item code.
    pub item_code: String,
    /// Partner category (Customer or Supplier).
    pub partner_type: PartnerType,
    /// Partner identifier (Customer ID or Supplier ID).
    pub partner_id: String,
    /// External part number / item code used by partner.
    pub partner_item_code: String,
    /// External description used on partner PO/Quote.
    pub partner_item_name: Option<String>,
}

/// Cross-Partner Reference Resolver for bidirectional lookup during transaction entry.
#[derive(Debug, Default, Clone)]
pub struct PartnerReferenceResolver {
    /// Key: `(PartnerType, PartnerID, PartnerItemCode)` -> Internal ItemCode
    partner_to_internal: HashMap<(PartnerType, String, String), String>,
    /// Key: `(PartnerType, PartnerID, InternalItemCode)` -> Partner ItemCode
    internal_to_partner: HashMap<(PartnerType, String, String), String>,
}

impl PartnerReferenceResolver {
    /// Creates a new empty partner reference resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a cross-partner reference.
    pub fn add_reference(&mut self, ref_entry: PartnerReference) {
        self.partner_to_internal.insert(
            (
                ref_entry.partner_type,
                ref_entry.partner_id.clone(),
                ref_entry.partner_item_code.clone(),
            ),
            ref_entry.item_code.clone(),
        );

        self.internal_to_partner.insert(
            (
                ref_entry.partner_type,
                ref_entry.partner_id,
                ref_entry.item_code,
            ),
            ref_entry.partner_item_code,
        );
    }

    /// Resolves an internal ERP item code from a partner's external part number.
    pub fn resolve_internal(
        &self,
        partner_type: PartnerType,
        partner_id: &str,
        partner_item_code: &str,
    ) -> Option<&str> {
        self.partner_to_internal
            .get(&(
                partner_type,
                partner_id.to_string(),
                partner_item_code.to_string(),
            ))
            .map(|s| s.as_str())
    }

    /// Resolves a partner's external part number from an internal ERP item code.
    pub fn resolve_partner(
        &self,
        partner_type: PartnerType,
        partner_id: &str,
        item_code: &str,
    ) -> Option<&str> {
        self.internal_to_partner
            .get(&(partner_type, partner_id.to_string(), item_code.to_string()))
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_reference_bidirectional_lookup() {
        let mut resolver = PartnerReferenceResolver::new();

        resolver.add_reference(PartnerReference {
            item_code: "INTERNAL-CAP-10UF".into(),
            partner_type: PartnerType::Supplier,
            partner_id: "DIGIKEY".into(),
            partner_item_code: "DK-CAP-10UF-0805-TR".into(),
            partner_item_name: Some("DigiKey 10uF 0805 Ceramic Capacitor".into()),
        });

        resolver.add_reference(PartnerReference {
            item_code: "INTERNAL-CAP-10UF".into(),
            partner_type: PartnerType::Customer,
            partner_id: "BOEING".into(),
            partner_item_code: "BAC-C10-001".into(),
            partner_item_name: Some("Boeing Aviation Spec Capacitor".into()),
        });

        // Resolve internal from DigiKey part number
        assert_eq!(
            resolver.resolve_internal(PartnerType::Supplier, "DIGIKEY", "DK-CAP-10UF-0805-TR"),
            Some("INTERNAL-CAP-10UF")
        );

        // Resolve Boeing customer part number from internal code
        assert_eq!(
            resolver.resolve_partner(PartnerType::Customer, "BOEING", "INTERNAL-CAP-10UF"),
            Some("BAC-C10-001")
        );
    }
}
