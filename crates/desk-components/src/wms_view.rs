//! Isometric 3D Warehouse Coordinate Grid Visualizer & Live AMR Tracking Model.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinViewModel {
    pub coordinate_code: String,
    pub zone: String,
    pub aisle: u16,
    pub bay: u16,
    pub level: u16,
    pub bin: u16,
    pub utilization_percent: f64,
    pub is_hazmat: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmrMarkerViewModel {
    pub robot_serial: String,
    pub x: f64,
    pub y: f64,
    pub status: String,
    pub battery_percent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Warehouse3DViewModel {
    pub warehouse_name: String,
    pub total_bins: usize,
    pub bins: Vec<BinViewModel>,
    pub robots: Vec<AmrMarkerViewModel>,
}

impl Warehouse3DViewModel {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            warehouse_name: name,
            total_bins: 0,
            bins: Vec::new(),
            robots: Vec::new(),
        }
    }

    pub fn add_bin(&mut self, bin: BinViewModel) {
        self.total_bins += 1;
        self.bins.push(bin);
    }

    pub fn update_robot(&mut self, robot: AmrMarkerViewModel) {
        if let Some(existing) = self
            .robots
            .iter_mut()
            .find(|r| r.robot_serial == robot.robot_serial)
        {
            *existing = robot;
        } else {
            self.robots.push(robot);
        }
    }
}
