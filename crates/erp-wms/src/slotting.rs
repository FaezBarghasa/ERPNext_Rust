//! Algorithmic Putaway Scoring & 3D Dynamic Slotting Optimizer.

use crate::grid::{HazmatClass, ThermalZone, WarehouseBin};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbcVelocity {
    ClassAHighVelocity, // Top 20% SKUs (80% of picks) -> Golden zone (0.8m - 1.4m height)
    ClassBMediumVelocity,
    ClassCLowVelocity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PutawayItem {
    pub sku: String,
    pub weight_kg: f64,
    pub volume_m3: f64,
    pub hazmat: HazmatClass,
    pub thermal: ThermalZone,
    pub velocity: AbcVelocity,
}

pub struct SlottingEngine;

impl SlottingEngine {
    /// Computes putaway fit score for a candidate bin (higher is better).
    #[must_use]
    pub fn score_bin(item: &PutawayItem, bin: &WarehouseBin) -> Option<f64> {
        if !bin.can_accommodate(item.weight_kg, item.volume_m3, &item.hazmat, &item.thermal) {
            return None;
        }

        // 1. Proximity to dock (lower aisle & bay number = closer)
        let distance_penalty = (bin.aisle as f64 * 10.0) + (bin.bay as f64 * 2.0);
        let dock_score = (100.0 - distance_penalty).max(0.0);

        // 2. ABC Velocity ergonomic height matching (Levels 2 & 3 are golden zones)
        let height_score = match (&item.velocity, bin.level) {
            (AbcVelocity::ClassAHighVelocity, 2 | 3) => 50.0,
            (AbcVelocity::ClassAHighVelocity, _) => 10.0,
            (AbcVelocity::ClassBMediumVelocity, 1 | 4) => 40.0,
            (AbcVelocity::ClassBMediumVelocity, _) => 20.0,
            (AbcVelocity::ClassCLowVelocity, 5 | 6) => 50.0,
            (AbcVelocity::ClassCLowVelocity, _) => 25.0,
        };

        // 3. Volumetric density utilization
        let remaining_vol = bin.max_volume_m3 - bin.current_volume_m3;
        let fit_ratio = (item.volume_m3 / remaining_vol).min(1.0);
        let fit_score = fit_ratio * 30.0;

        Some(dock_score + height_score + fit_score)
    }

    /// Selects optimal candidate bin from a list.
    pub fn select_best_bin<'a>(
        item: &PutawayItem,
        bins: &'a [WarehouseBin],
    ) -> Option<(&'a WarehouseBin, f64)> {
        let mut best: Option<(&'a WarehouseBin, f64)> = None;
        for bin in bins {
            if let Some(score) = Self::score_bin(item, bin) {
                if best.is_none() || score > best.unwrap().1 {
                    best = Some((bin, score));
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slotting_score_selection() {
        let item = PutawayItem {
            sku: "SKU-FAST-01".into(),
            weight_kg: 5.0,
            volume_m3: 0.05,
            hazmat: HazmatClass::NonHazmat,
            thermal: ThermalZone::Ambient,
            velocity: AbcVelocity::ClassAHighVelocity,
        };

        let bins = vec![
            WarehouseBin {
                id: "BIN-FAR-TOP".into(),
                site: "S1".into(),
                warehouse: "W1".into(),
                zone: "Z1".into(),
                aisle: 10,
                bay: 20,
                level: 6,
                bin: 1,
                thermal_zone: ThermalZone::Ambient,
                max_weight_kg: 500.0,
                max_volume_m3: 1.0,
                current_weight_kg: 0.0,
                current_volume_m3: 0.0,
                allowed_hazmat: vec![],
            },
            WarehouseBin {
                id: "BIN-NEAR-GOLDEN".into(),
                site: "S1".into(),
                warehouse: "W1".into(),
                zone: "Z1".into(),
                aisle: 1,
                bay: 2,
                level: 2, // Golden level
                bin: 1,
                thermal_zone: ThermalZone::Ambient,
                max_weight_kg: 500.0,
                max_volume_m3: 1.0,
                current_weight_kg: 0.0,
                current_volume_m3: 0.0,
                allowed_hazmat: vec![],
            },
        ];

        let (best_bin, score) = SlottingEngine::select_best_bin(&item, &bins).unwrap();
        assert_eq!(best_bin.id, "BIN-NEAR-GOLDEN");
        assert!(score > 100.0);
    }
}
