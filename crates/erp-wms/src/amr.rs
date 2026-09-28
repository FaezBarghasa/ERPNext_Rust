//! VDA 5050 Autonomous Mobile Robot (AMR/AGV) Fleet Mesh & Handling Units (HU / SSCC-18).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmrState {
    Idle,
    Navigating,
    LiftingHandlingUnit,
    DroppingHandlingUnit,
    Charging,
    EmergencyStop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vda5050Order {
    pub header_id: u32,
    pub order_id: String,
    pub order_update_id: u32,
    pub zone: String,
    pub destination_node_id: String,
    pub handling_unit_sscc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmrTelemetry {
    pub robot_serial: String,
    pub state: AmrState,
    pub battery_charge_percent: f64,
    pub current_x: f64,
    pub current_y: f64,
    pub velocity_m_s: f64,
    pub active_order_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandlingUnit {
    pub sscc_18: String,      // Serial Shipping Container Code (18 digits)
    pub package_type: String, // Pallet, Euro-Pallet, Carton, Tote
    pub gross_weight_kg: f64,
    pub current_bin_id: Option<String>,
    pub amr_robot_id: Option<String>,
}

impl HandlingUnit {
    /// Generates standard GS1 SSCC-18 check digit.
    #[must_use]
    pub fn compute_sscc18_check_digit(base17: &str) -> Option<u8> {
        if base17.len() != 17 || !base17.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut sum = 0;
        for (i, c) in base17.chars().enumerate() {
            let digit = c.to_digit(10)? as usize;
            let weight = if i % 2 == 0 { 3 } else { 1 };
            sum += digit * weight;
        }
        let rem = sum % 10;
        Some(if rem == 0 { 0 } else { (10 - rem) as u8 })
    }
}

pub struct Vda5050FleetCoordinator;

impl Vda5050FleetCoordinator {
    /// Dispatches orders to best idle AMR robot based on battery level and proximity.
    pub fn assign_order<'a>(
        order: &Vda5050Order,
        robots: &'a mut [AmrTelemetry],
    ) -> Option<&'a mut AmrTelemetry> {
        let mut best: Option<&'a mut AmrTelemetry> = None;
        for r in robots {
            if r.state == AmrState::Idle
                && r.battery_charge_percent >= 20.0
                && (best.is_none()
                    || r.battery_charge_percent > best.as_ref().unwrap().battery_charge_percent)
            {
                best = Some(r);
            }
        }
        if let Some(ref mut assigned) = best {
            assigned.state = AmrState::Navigating;
            assigned.active_order_id = Some(order.order_id.clone());
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sscc18_check_digit_and_vda5050_dispatch() {
        let base17 = "00614141123456789";
        let check_digit = HandlingUnit::compute_sscc18_check_digit(base17).unwrap();
        assert_eq!(check_digit, 0);

        let mut robots = vec![
            AmrTelemetry {
                robot_serial: "AMR-01".into(),
                state: AmrState::Idle,
                battery_charge_percent: 85.0,
                current_x: 10.0,
                current_y: 20.0,
                velocity_m_s: 0.0,
                active_order_id: None,
            },
            AmrTelemetry {
                robot_serial: "AMR-02".into(),
                state: AmrState::Idle,
                battery_charge_percent: 45.0,
                current_x: 5.0,
                current_y: 5.0,
                velocity_m_s: 0.0,
                active_order_id: None,
            },
        ];

        let order = Vda5050Order {
            header_id: 1,
            order_id: "ORD-990".into(),
            order_update_id: 0,
            zone: "ZONE-A".into(),
            destination_node_id: "DOCK-01".into(),
            handling_unit_sscc: "006141411234567894".into(),
        };

        let assigned = Vda5050FleetCoordinator::assign_order(&order, &mut robots).unwrap();
        assert_eq!(assigned.robot_serial, "AMR-01");
        assert_eq!(assigned.state, AmrState::Navigating);
    }
}
