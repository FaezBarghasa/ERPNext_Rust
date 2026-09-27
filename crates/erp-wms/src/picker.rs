//! Dynamic Traveling Salesperson (TSP) Pick-Path Optimizer for Warehouse Logistics.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PickLocation {
    pub id: String,
    pub sku: String,
    pub x: f64, // Euclidean X coordinate (meters from depot)
    pub y: f64, // Euclidean Y coordinate (meters from depot)
    pub z: f64, // Shelf height (meters)
    pub quantity: u32,
}

impl PickLocation {
    #[must_use]
    pub fn distance_to(&self, other: &PickLocation) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = (self.z - other.z) * 1.5; // Height penalty multiplier
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

pub struct PickPathOptimizer;

impl PickPathOptimizer {
    /// Computes TSP route using Nearest Neighbor with 2-Opt local search refinement.
    pub fn optimize_pick_route(
        depot: &PickLocation,
        picks: &[PickLocation],
    ) -> (Vec<PickLocation>, f64) {
        if picks.is_empty() {
            return (vec![], 0.0);
        }

        let mut unvisited: Vec<PickLocation> = picks.to_vec();
        let mut route: Vec<PickLocation> = Vec::with_capacity(picks.len() + 2);
        let mut current = depot.clone();
        route.push(depot.clone());

        // 1. Nearest Neighbor construction
        while !unvisited.is_empty() {
            let mut nearest_idx = 0;
            let mut min_dist = f64::MAX;

            for (idx, loc) in unvisited.iter().enumerate() {
                let dist = current.distance_to(loc);
                if dist < min_dist {
                    min_dist = dist;
                    nearest_idx = idx;
                }
            }

            let next_loc = unvisited.remove(nearest_idx);
            current = next_loc.clone();
            route.push(next_loc);
        }
        route.push(depot.clone()); // Return to depot

        // 2. 2-Opt heuristic refinement
        let n = route.len();
        let mut improved = true;
        let mut iterations = 0;

        while improved && iterations < 50 {
            improved = false;
            iterations += 1;
            for i in 1..(n - 2) {
                for j in (i + 1)..(n - 1) {
                    let d_before =
                        route[i - 1].distance_to(&route[i]) + route[j].distance_to(&route[j + 1]);
                    let d_after =
                        route[i - 1].distance_to(&route[j]) + route[i].distance_to(&route[j + 1]);
                    if d_after < d_before - 1e-4 {
                        route[i..=j].reverse();
                        improved = true;
                    }
                }
            }
        }

        // Calculate total path distance
        let mut total_distance = 0.0;
        for i in 0..(route.len() - 1) {
            total_distance += route[i].distance_to(&route[i + 1]);
        }

        (route, total_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsp_pick_path_optimization() {
        let depot = PickLocation {
            id: "DEPOT".into(),
            sku: "".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            quantity: 0,
        };

        let picks = vec![
            PickLocation {
                id: "P1".into(),
                sku: "SKU1".into(),
                x: 10.0,
                y: 50.0,
                z: 1.0,
                quantity: 2,
            },
            PickLocation {
                id: "P2".into(),
                sku: "SKU2".into(),
                x: 10.0,
                y: 10.0,
                z: 1.0,
                quantity: 1,
            },
            PickLocation {
                id: "P3".into(),
                sku: "SKU3".into(),
                x: 20.0,
                y: 15.0,
                z: 1.0,
                quantity: 5,
            },
            PickLocation {
                id: "P4".into(),
                sku: "SKU4".into(),
                x: 20.0,
                y: 45.0,
                z: 1.0,
                quantity: 3,
            },
        ];

        let (route, total_dist) = PickPathOptimizer::optimize_pick_route(&depot, &picks);
        assert_eq!(route.len(), 6); // Depot + 4 picks + Depot
        assert!(total_dist < 150.0);
    }
}
