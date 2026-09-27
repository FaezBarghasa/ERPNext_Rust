//! Linear Referencing System (LRS) & GIS Spatial Topology for Continuous Non-Discrete Assets.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinearMarker {
    pub marker_id: String,
    pub name: String,
    pub route_id: String,
    pub chainage_meters: f64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinearSegment {
    pub segment_id: String,
    pub asset_id: String,
    pub start_chainage: f64,
    pub end_chainage: f64,
    pub material_spec: String,
    pub operating_pressure_psi: Option<f64>,
}

impl LinearSegment {
    #[must_use]
    pub fn length_meters(&self) -> f64 {
        (self.end_chainage - self.start_chainage).abs()
    }

    #[must_use]
    pub fn contains_chainage(&self, chainage: f64) -> bool {
        chainage >= self.start_chainage && chainage <= self.end_chainage
    }
}

pub struct LrsEngine;

impl LrsEngine {
    #[must_use]
    pub fn interpolate_coordinates(
        start_marker: &LinearMarker,
        end_marker: &LinearMarker,
        target_chainage: f64,
    ) -> Option<(f64, f64)> {
        if target_chainage < start_marker.chainage_meters || target_chainage > end_marker.chainage_meters {
            return None;
        }
        let total_dist = end_marker.chainage_meters - start_marker.chainage_meters;
        if total_dist == 0.0 {
            return Some((start_marker.latitude, start_marker.longitude));
        }

        let ratio = (target_chainage - start_marker.chainage_meters) / total_dist;
        let lat = start_marker.latitude + ratio * (end_marker.latitude - start_marker.latitude);
        let lon = start_marker.longitude + ratio * (end_marker.longitude - start_marker.longitude);
        Some((lat, lon))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrs_interpolation() {
        let m1 = LinearMarker {
            marker_id: "MP-100".into(),
            name: "Milepost 100".into(),
            route_id: "PIPELINE-ALPHA".into(),
            chainage_meters: 1000.0,
            latitude: 35.000,
            longitude: 51.000,
        };
        let m2 = LinearMarker {
            marker_id: "MP-101".into(),
            name: "Milepost 101".into(),
            route_id: "PIPELINE-ALPHA".into(),
            chainage_meters: 2000.0,
            latitude: 35.100,
            longitude: 51.100,
        };

        let (lat, lon) = LrsEngine::interpolate_coordinates(&m1, &m2, 1500.0).unwrap();
        assert!((lat - 35.050).abs() < 1e-5);
        assert!((lon - 51.050).abs() < 1e-5);
    }
}
