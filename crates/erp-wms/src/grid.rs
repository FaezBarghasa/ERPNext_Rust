//! Multi-Tier Spatial Location Topology & Hazmat Chemical Co-Storage Matrix.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThermalZone {
    Ambient,
    Chilled2To8C,
    CryogenicMinus80C,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HazmatClass {
    Class1Explosives,
    Class2Gases,
    Class3FlammableLiquids,
    Class4FlammableSolids,
    Class5Oxidizers,
    Class6Toxics,
    Class7Radioactive,
    Class8Corrosives,
    Class9Miscellaneous,
    NonHazmat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarehouseBin {
    pub id: String,
    pub site: String,
    pub warehouse: String,
    pub zone: String,
    pub aisle: u16,
    pub bay: u16,
    pub level: u16,
    pub bin: u16,
    pub thermal_zone: ThermalZone,
    pub max_weight_kg: f64,
    pub max_volume_m3: f64,
    pub current_weight_kg: f64,
    pub current_volume_m3: f64,
    pub allowed_hazmat: Vec<HazmatClass>,
}

impl WarehouseBin {
    #[must_use]
    pub fn coordinate_code(&self) -> String {
        format!("{}-{}-{:02}-{:02}-{:02}-{:02}", self.zone, self.aisle, self.bay, self.level, self.bin, 0)
    }

    #[must_use]
    pub fn can_accommodate(&self, weight: f64, volume: f64, hazmat: &HazmatClass, temp: &ThermalZone) -> bool {
        if &self.thermal_zone != temp {
            return false;
        }
        if self.current_weight_kg + weight > self.max_weight_kg {
            return false;
        }
        if self.current_volume_m3 + volume > self.max_volume_m3 {
            return false;
        }
        if hazmat != &HazmatClass::NonHazmat && !self.allowed_hazmat.contains(hazmat) {
            return false;
        }
        true
    }
}

pub struct HazmatMatrix;

impl HazmatMatrix {
    /// Evaluates if two hazard classes can be co-stored within the same aisle or drainage basin.
    #[must_use]
    pub fn are_compatible(a: &HazmatClass, b: &HazmatClass) -> bool {
        if a == &HazmatClass::NonHazmat || b == &HazmatClass::NonHazmat {
            return true;
        }
        if a == b {
            return true;
        }

        // Hard rule: Class 3 (Flammable Liquids) incompatible with Class 5 (Oxidizers)
        if (a == &HazmatClass::Class3FlammableLiquids && b == &HazmatClass::Class5Oxidizers)
            || (a == &HazmatClass::Class5Oxidizers && b == &HazmatClass::Class3FlammableLiquids)
        {
            return false;
        }

        // Hard rule: Class 8 (Corrosives) incompatible with Class 4 (Flammable Solids)
        if (a == &HazmatClass::Class8Corrosives && b == &HazmatClass::Class4FlammableSolids)
            || (a == &HazmatClass::Class4FlammableSolids && b == &HazmatClass::Class8Corrosives)
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_capacity_and_hazmat_segregation() {
        let bin = WarehouseBin {
            id: "BIN-01".into(),
            site: "SITE-1".into(),
            warehouse: "MAIN".into(),
            zone: "ZONE-A".into(),
            aisle: 1,
            bay: 4,
            level: 2,
            bin: 1,
            thermal_zone: ThermalZone::Ambient,
            max_weight_kg: 1000.0,
            max_volume_m3: 2.5,
            current_weight_kg: 200.0,
            current_volume_m3: 0.5,
            allowed_hazmat: vec![HazmatClass::Class3FlammableLiquids],
        };

        assert!(bin.can_accommodate(100.0, 0.2, &HazmatClass::Class3FlammableLiquids, &ThermalZone::Ambient));
        assert!(!bin.can_accommodate(900.0, 0.2, &HazmatClass::Class3FlammableLiquids, &ThermalZone::Ambient)); // Exceeds weight
        assert!(!bin.can_accommodate(100.0, 0.2, &HazmatClass::Class5Oxidizers, &ThermalZone::Ambient)); // Hazmat not allowed

        assert!(!HazmatMatrix::are_compatible(&HazmatClass::Class3FlammableLiquids, &HazmatClass::Class5Oxidizers));
    }
}
