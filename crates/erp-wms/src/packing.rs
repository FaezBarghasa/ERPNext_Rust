//! 3D Container & Pallet Volumetric Bin Packing Solver (Milestone 8.1).
//! Computes optimal 3D item placement inside pallets/containers to maximize space utilization.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackingBox {
    pub id: String,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub weight_kg: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlacedItem {
    pub box_id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub weight_kg: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalletBin {
    pub id: String,
    pub max_width: f64,
    pub max_height: f64,
    pub max_depth: f64,
    pub max_weight_kg: f64,
    pub placed_items: Vec<PlacedItem>,
    pub total_weight_kg: f64,
}

impl PalletBin {
    #[must_use]
    pub fn new(id: String, width: f64, height: f64, depth: f64, max_weight: f64) -> Self {
        Self {
            id,
            max_width: width,
            max_height: height,
            max_depth: depth,
            max_weight_kg: max_weight,
            placed_items: Vec::new(),
            total_weight_kg: 0.0,
        }
    }

    #[must_use]
    pub fn volume(&self) -> f64 {
        self.max_width * self.max_height * self.max_depth
    }

    #[must_use]
    pub fn utilized_volume(&self) -> f64 {
        self.placed_items
            .iter()
            .map(|item| item.width * item.height * item.depth)
            .sum()
    }

    #[must_use]
    pub fn volumetric_efficiency_percent(&self) -> f64 {
        let vol = self.volume();
        if vol <= 0.0 {
            0.0
        } else {
            (self.utilized_volume() / vol) * 100.0
        }
    }
}

pub struct BinPacking3DSolver;

impl BinPacking3DSolver {
    /// Packs a set of boxes into a pallet bin using greedy bottom-left-first heuristic.
    pub fn pack_pallet(bin: &mut PalletBin, mut boxes: Vec<PackingBox>) -> Vec<PackingBox> {
        // Sort boxes descending by volume to pack large items first
        boxes.sort_by(|a, b| {
            let vol_a = a.width * a.height * a.depth;
            let vol_b = b.width * b.height * b.depth;
            vol_b
                .partial_cmp(&vol_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut unplaced = Vec::new();

        for b in boxes {
            if bin.total_weight_kg + b.weight_kg > bin.max_weight_kg {
                unplaced.push(b);
                continue;
            }

            // Find placement candidate coordinate
            let mut placed = false;
            let mut candidate_z = 0.0;

            while candidate_z + b.height <= bin.max_height {
                let mut candidate_y = 0.0;
                while candidate_y + b.depth <= bin.max_depth {
                    let mut candidate_x = 0.0;
                    while candidate_x + b.width <= bin.max_width {
                        // Check collision with already placed items
                        let collides = bin.placed_items.iter().any(|p| {
                            candidate_x < p.x + p.width
                                && candidate_x + b.width > p.x
                                && candidate_y < p.y + p.depth
                                && candidate_y + b.depth > p.y
                                && candidate_z < p.z + p.height
                                && candidate_z + b.height > p.z
                        });

                        if !collides {
                            bin.placed_items.push(PlacedItem {
                                box_id: b.id.clone(),
                                x: candidate_x,
                                y: candidate_y,
                                z: candidate_z,
                                width: b.width,
                                height: b.height,
                                depth: b.depth,
                                weight_kg: b.weight_kg,
                            });
                            bin.total_weight_kg += b.weight_kg;
                            placed = true;
                            break;
                        }
                        candidate_x += 10.0; // Step resolution
                    }
                    if placed {
                        break;
                    }
                    candidate_y += 10.0;
                }
                if placed {
                    break;
                }
                candidate_z += 10.0;
            }

            if !placed {
                unplaced.push(b);
            }
        }

        unplaced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3d_bin_packing_solver() {
        let mut euro_pallet = PalletBin::new("EURO-PALLET-01".into(), 120.0, 160.0, 80.0, 1000.0);
        let boxes = vec![
            PackingBox {
                id: "BOX-A1".into(),
                width: 40.0,
                height: 40.0,
                depth: 40.0,
                weight_kg: 25.0,
            },
            PackingBox {
                id: "BOX-A2".into(),
                width: 40.0,
                height: 40.0,
                depth: 40.0,
                weight_kg: 25.0,
            },
            PackingBox {
                id: "BOX-B1".into(),
                width: 60.0,
                height: 40.0,
                depth: 40.0,
                weight_kg: 50.0,
            },
        ];

        let unplaced = BinPacking3DSolver::pack_pallet(&mut euro_pallet, boxes);
        assert!(unplaced.is_empty());
        assert_eq!(euro_pallet.placed_items.len(), 3);
        assert!(euro_pallet.volumetric_efficiency_percent() > 5.0);
        assert_eq!(euro_pallet.total_weight_kg, 100.0);
    }
}
