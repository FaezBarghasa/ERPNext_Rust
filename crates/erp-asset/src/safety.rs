//! Cryptographic Safety Interlocks, Permit-to-Work (PTW) & Lockout/Tagout (LOTO).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IsolationPointType {
    ElectricalBreaker,
    ValveMechanical,
    PneumaticLine,
    ChemicalFlange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LotoTag {
    pub isolation_id: String,
    pub point_type: IsolationPointType,
    pub location: String,
    pub padlock_serial: String,
    pub marshal_user_id: String,
    pub is_locked: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermitToWork {
    pub permit_id: String,
    pub asset_id: String,
    pub work_order_id: String,
    pub work_type: String, // Hot Work, Confined Space, High Voltage, Radiation
    pub loto_tags: Vec<LotoTag>,
    pub gas_test_passed: bool,
    pub safety_marshal_signature: Option<String>,
    pub is_active: bool,
}

impl PermitToWork {
    #[must_use]
    pub fn new(
        permit_id: String,
        asset_id: String,
        work_order_id: String,
        work_type: String,
    ) -> Self {
        Self {
            permit_id,
            asset_id,
            work_order_id,
            work_type,
            loto_tags: Vec::new(),
            gas_test_passed: false,
            safety_marshal_signature: None,
            is_active: false,
        }
    }

    pub fn add_loto_tag(&mut self, tag: LotoTag) {
        self.loto_tags.push(tag);
    }

    /// Validates all cryptographic and physical isolation prerequisites before authorizing work commencement.
    pub fn authorize(&mut self, marshal_id: &str, secret_key: &str) -> Result<String, String> {
        if self.loto_tags.is_empty() {
            return Err(
                "Cannot authorize PTW: zero LOTO physical isolation points attached".into(),
            );
        }

        let all_locked = self.loto_tags.iter().all(|t| t.is_locked);
        if !all_locked {
            return Err(
                "Cannot authorize PTW: one or more LOTO isolation points are UNLOCKED".into(),
            );
        }

        if self.work_type.contains("Confined") && !self.gas_test_passed {
            return Err(
                "Cannot authorize PTW: Confined space requires atmospheric gas sniff test pass"
                    .into(),
            );
        }

        // Generate tamper-evident cryptographic marshal signature
        let mut hasher = Sha256::new();
        hasher.update(self.permit_id.as_bytes());
        hasher.update(self.work_order_id.as_bytes());
        hasher.update(marshal_id.as_bytes());
        hasher.update(secret_key.as_bytes());
        let sig: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        self.safety_marshal_signature = Some(sig.clone());
        self.is_active = true;
        Ok(sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ptw_loto_authorization_lifecycle() {
        let mut ptw = PermitToWork::new(
            "PTW-2026-089".into(),
            "TURBINE-01".into(),
            "WO-9912".into(),
            "High Voltage Repair".into(),
        );

        ptw.add_loto_tag(LotoTag {
            isolation_id: "ISO-BREAKER-4160V".into(),
            point_type: IsolationPointType::ElectricalBreaker,
            location: "Substation Bay 4".into(),
            padlock_serial: "PADLOCK-991".into(),
            marshal_user_id: "marshal_bob".into(),
            is_locked: true,
            timestamp: chrono::Utc::now(),
        });

        let sig = ptw.authorize("marshal_bob", "vault_secret_token").unwrap();
        assert!(!sig.is_empty());
        assert!(ptw.is_active);
    }
}
