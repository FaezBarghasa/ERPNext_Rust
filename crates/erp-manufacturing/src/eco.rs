//! Engineering Change Order (ECO) State Machine & Stock Disposition Engine.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispositionMode {
    ScrapImmediately,
    ReworkToRevNext { rework_routing_id: String },
    RunOutExistingStock { max_units: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcoStatus {
    Draft,
    EngineeringReview,
    QualityReview,
    Approved,
    Implemented,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineeringChangeOrder {
    pub eco_number: String,
    pub item_code: String,
    pub current_revision: String,
    pub target_revision: String,
    pub reason_for_change: String,
    pub disposition: DispositionMode,
    pub status: EcoStatus,
    pub effective_date: chrono::NaiveDate,
    pub approvals: Vec<String>, // Approver user signatures
}

impl EngineeringChangeOrder {
    #[must_use]
    pub fn new(
        eco_number: String,
        item_code: String,
        current_rev: String,
        target_rev: String,
        reason: String,
        disposition: DispositionMode,
        effective_date: chrono::NaiveDate,
    ) -> Self {
        Self {
            eco_number,
            item_code,
            current_revision: current_rev,
            target_revision: target_rev,
            reason_for_change: reason,
            disposition,
            status: EcoStatus::Draft,
            effective_date,
            approvals: Vec::new(),
        }
    }

    pub fn approve(&mut self, approver_id: &str) -> Result<(), String> {
        if self.status == EcoStatus::Implemented || self.status == EcoStatus::Cancelled {
            return Err("Cannot approve an implemented or cancelled ECO".into());
        }
        self.approvals.push(approver_id.to_string());
        if self.approvals.len() >= 2 {
            self.status = EcoStatus::Approved;
        }
        Ok(())
    }

    pub fn implement(&mut self) -> Result<(), String> {
        if self.status != EcoStatus::Approved {
            return Err("ECO must be Approved before implementation".into());
        }
        self.status = EcoStatus::Implemented;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eco_approval_lifecycle() {
        let mut eco = EngineeringChangeOrder::new(
            "ECO-2026-004".into(),
            "ROTOR-BLADE".into(),
            "Rev A".into(),
            "Rev B".into(),
            "Aerodynamic efficiency improvement".into(),
            DispositionMode::RunOutExistingStock { max_units: 50 },
            chrono::NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        );

        assert_eq!(eco.status, EcoStatus::Draft);
        eco.approve("eng_lead").unwrap();
        assert_eq!(eco.status, EcoStatus::Draft); // Requires 2 approvals
        eco.approve("qa_director").unwrap();
        assert_eq!(eco.status, EcoStatus::Approved);
        eco.implement().unwrap();
        assert_eq!(eco.status, EcoStatus::Implemented);
    }
}
