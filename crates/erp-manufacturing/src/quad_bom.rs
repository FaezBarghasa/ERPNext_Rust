//! Quad-BOM Synchronization Engine (EBOM, MBOM, SBOM) & Divergence Diffing.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BomType {
    EngineeringBom,  // EBOM (Functional/CAD hierarchy)
    ManufacturingBom, // MBOM (Routing, operations, consumable phantoms)
    ServiceBom,       // SBOM (Field-replaceable units, wear kits)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BomLineItem {
    pub item_code: String,
    pub description: String,
    pub quantity: Decimal,
    pub uom: String,
    pub is_field_replaceable: bool,
    pub is_phantom_subassembly: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructuredBom {
    pub bom_id: String,
    pub parent_product: String,
    pub bom_type: BomType,
    pub revision: String,
    pub items: Vec<BomLineItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BomDivergence {
    pub item_code: String,
    pub ebom_qty: Option<Decimal>,
    pub mbom_qty: Option<Decimal>,
    pub sbom_qty: Option<Decimal>,
    pub reason: String,
}

pub struct QuadBomSynchronizer;

impl QuadBomSynchronizer {
    /// Compares EBOM, MBOM, and SBOM to identify unmapped engineering deviations.
    pub fn diff_boms(
        ebom: &StructuredBom,
        mbom: &StructuredBom,
        sbom: Option<&StructuredBom>,
    ) -> Vec<BomDivergence> {
        let mut e_map: HashMap<String, Decimal> = HashMap::new();
        for item in &ebom.items {
            *e_map.entry(item.item_code.clone()).or_insert(Decimal::ZERO) += item.quantity;
        }

        let mut m_map: HashMap<String, Decimal> = HashMap::new();
        for item in &mbom.items {
            *m_map.entry(item.item_code.clone()).or_insert(Decimal::ZERO) += item.quantity;
        }

        let mut s_map: HashMap<String, Decimal> = HashMap::new();
        if let Some(s) = sbom {
            for item in &s.items {
                *s_map.entry(item.item_code.clone()).or_insert(Decimal::ZERO) += item.quantity;
            }
        }

        let mut all_keys: std::collections::HashSet<String> = e_map.keys().cloned().collect();
        all_keys.extend(m_map.keys().cloned());
        all_keys.extend(s_map.keys().cloned());

        let mut divergences = Vec::new();
        for key in all_keys {
            let eq = e_map.get(&key).copied();
            let mq = m_map.get(&key).copied();
            let sq = s_map.get(&key).copied();

            if eq != mq {
                divergences.push(BomDivergence {
                    item_code: key.clone(),
                    ebom_qty: eq,
                    mbom_qty: mq,
                    sbom_qty: sq,
                    reason: if eq.is_none() {
                        "Present in MBOM as operational consumable/phantom".into()
                    } else if mq.is_none() {
                        "In EBOM but missing from manufacturing routing".into()
                    } else {
                        "Quantity discrepancy between EBOM and MBOM".into()
                    },
                });
            }
        }

        divergences
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_quad_bom_diffing() {
        let ebom = StructuredBom {
            bom_id: "EBOM-01".into(),
            parent_product: "TURBINE-X".into(),
            bom_type: BomType::EngineeringBom,
            revision: "A".into(),
            items: vec![BomLineItem {
                item_code: "ROTOR-BLADE".into(),
                description: "Titanium Blade".into(),
                quantity: dec!(24),
                uom: "Nos".into(),
                is_field_replaceable: true,
                is_phantom_subassembly: false,
            }],
        };

        let mbom = StructuredBom {
            bom_id: "MBOM-01".into(),
            parent_product: "TURBINE-X".into(),
            bom_type: BomType::ManufacturingBom,
            revision: "A.1".into(),
            items: vec![
                BomLineItem {
                    item_code: "ROTOR-BLADE".into(),
                    description: "Titanium Blade".into(),
                    quantity: dec!(24),
                    uom: "Nos".into(),
                    is_field_replaceable: true,
                    is_phantom_subassembly: false,
                },
                BomLineItem {
                    item_code: "LOCTITE-680".into(),
                    description: "Retaining Compound".into(),
                    quantity: dec!(0.05),
                    uom: "Kg".into(),
                    is_field_replaceable: false,
                    is_phantom_subassembly: true,
                },
            ],
        };

        let diffs = QuadBomSynchronizer::diff_boms(&ebom, &mbom, None);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].item_code, "LOCTITE-680");
    }
}
