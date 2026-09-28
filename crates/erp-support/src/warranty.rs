//! Warranty Claims, Maintenance Schedules, and Maintenance Visits Governance.
//!
//! Provides:
//! - Warranty eligibility verification for serialized and batched items.
//! - Preventive maintenance schedule generation based on periodicity.
//! - Maintenance visit checklists, technician logs, and customer sign-off capture.

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors arising from warranty validation and maintenance scheduling.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WarrantyError {
    #[error("Item {serial_no} warranty expired on {expiry_date}")]
    WarrantyExpired {
        serial_no: String,
        expiry_date: String,
    },
    #[error("No active maintenance contract found for customer {customer_id}")]
    ContractNotFound { customer_id: String },
    #[error("Maintenance visit {visit_id} already completed")]
    VisitAlreadyCompleted { visit_id: String },
}

/// Status of a warranty claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarrantyStatus {
    UnderWarranty,
    OutofWarranty,
    ClaimSubmitted,
    ClaimApproved,
    ClaimRejected,
    Replaced,
}

/// A registered warranty claim for an equipment/serial number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarrantyClaim {
    pub claim_id: String,
    pub customer_id: String,
    pub item_code: String,
    pub serial_no: String,
    pub sale_date: DateTime<Utc>,
    pub warranty_period_days: u32,
    pub issue_description: String,
    pub status: WarrantyStatus,
    pub resolution_notes: Option<String>,
}

impl WarrantyClaim {
    /// Evaluates if the serial number is currently eligible under active warranty.
    pub fn is_eligible(&self, claim_date: DateTime<Utc>) -> Result<bool, WarrantyError> {
        let expiry = self.sale_date + Duration::days(i64::from(self.warranty_period_days));
        if claim_date > expiry {
            Err(WarrantyError::WarrantyExpired {
                serial_no: self.serial_no.clone(),
                expiry_date: expiry.format("%Y-%m-%d").to_string(),
            })
        } else {
            Ok(true)
        }
    }
}

/// Periodicity for scheduled preventive maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenancePeriodicity {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

/// An individual scheduled visit task within a maintenance plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMaintenanceEvent {
    pub event_id: String,
    pub scheduled_date: DateTime<Utc>,
    pub technician_id: Option<String>,
    pub completed: bool,
}

/// A multi-period maintenance schedule contract for a customer site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSchedule {
    pub schedule_id: String,
    pub customer_id: String,
    pub item_code: String,
    pub serial_no: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub periodicity: MaintenancePeriodicity,
    pub events: Vec<ScheduledMaintenanceEvent>,
}

impl MaintenanceSchedule {
    /// Generates periodic preventive maintenance events between start and end dates.
    pub fn generate_events(
        schedule_id: &str,
        customer_id: &str,
        item_code: &str,
        serial_no: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        periodicity: MaintenancePeriodicity,
    ) -> Self {
        let mut events = Vec::new();
        let mut current = start_date;
        let mut idx = 1;

        while current <= end_date {
            events.push(ScheduledMaintenanceEvent {
                event_id: format!("{schedule_id}-EV-{idx:02}"),
                scheduled_date: current,
                technician_id: None,
                completed: false,
            });
            idx += 1;

            current = match periodicity {
                MaintenancePeriodicity::Monthly => {
                    let next_month = if current.month() == 12 {
                        1
                    } else {
                        current.month() + 1
                    };
                    let next_year = if current.month() == 12 {
                        current.year() + 1
                    } else {
                        current.year()
                    };
                    current
                        .with_year(next_year)
                        .and_then(|d| d.with_month(next_month))
                        .unwrap_or(current + Duration::days(30))
                }
                MaintenancePeriodicity::Quarterly => current + Duration::days(90),
                MaintenancePeriodicity::HalfYearly => current + Duration::days(180),
                MaintenancePeriodicity::Yearly => current + Duration::days(365),
            };
        }

        Self {
            schedule_id: schedule_id.to_string(),
            customer_id: customer_id.to_string(),
            item_code: item_code.to_string(),
            serial_no: serial_no.to_string(),
            start_date,
            end_date,
            periodicity,
            events,
        }
    }
}

/// A checklist item evaluated during a technician's maintenance visit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceChecklistItem {
    pub parameter_name: String,
    pub is_passed: bool,
    pub observation: String,
}

/// An on-site technician maintenance visit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceVisit {
    pub visit_id: String,
    pub schedule_id: Option<String>,
    pub customer_id: String,
    pub technician_id: String,
    pub visit_date: DateTime<Utc>,
    pub checklist: Vec<MaintenanceChecklistItem>,
    pub customer_signature: Option<String>,
    pub completed: bool,
}

impl MaintenanceVisit {
    /// Completes the visit with a customer sign-off.
    pub fn complete_visit(&mut self, signature: &str) -> Result<(), WarrantyError> {
        if self.completed {
            return Err(WarrantyError::VisitAlreadyCompleted {
                visit_id: self.visit_id.clone(),
            });
        }
        self.customer_signature = Some(signature.to_string());
        self.completed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warranty_eligibility_and_expiration() {
        let sale_date = Utc::now() - Duration::days(100);
        let claim = WarrantyClaim {
            claim_id: "WC-001".into(),
            customer_id: "CUST-ACME".into(),
            item_code: "PUMP-X1".into(),
            serial_no: "SN-9821".into(),
            sale_date,
            warranty_period_days: 365,
            issue_description: "Bearing noise".into(),
            status: WarrantyStatus::ClaimSubmitted,
            resolution_notes: None,
        };

        // Within 365 days -> Eligible
        assert!(claim.is_eligible(Utc::now()).is_ok());

        // Evaluation after 400 days -> Expired error
        let future_claim = Utc::now() + Duration::days(300);
        assert!(matches!(
            claim.is_eligible(future_claim),
            Err(WarrantyError::WarrantyExpired { .. })
        ));
    }

    #[test]
    fn test_maintenance_schedule_generation() {
        let start = Utc::now();
        let end = start + Duration::days(365);
        let schedule = MaintenanceSchedule::generate_events(
            "MS-2026-01",
            "CUST-BETA",
            "HVAC-UNIT",
            "SN-HVAC-09",
            start,
            end,
            MaintenancePeriodicity::Quarterly,
        );

        // Quarterly in 365 days yields 5 events (0, 90, 180, 270, 360 days)
        assert_eq!(schedule.events.len(), 5);
        assert_eq!(schedule.events[0].event_id, "MS-2026-01-EV-01");
        assert!(!schedule.events[0].completed);
    }

    #[test]
    fn test_maintenance_visit_completion_signoff() {
        let mut visit = MaintenanceVisit {
            visit_id: "MV-2026-101".into(),
            schedule_id: Some("MS-2026-01".into()),
            customer_id: "CUST-BETA".into(),
            technician_id: "TECH-ALEX".into(),
            visit_date: Utc::now(),
            checklist: vec![MaintenanceChecklistItem {
                parameter_name: "Compressor Pressure".into(),
                is_passed: true,
                observation: "Normal 65 PSI".into(),
            }],
            customer_signature: None,
            completed: false,
        };

        assert!(
            visit
                .complete_visit("data:image/svg+xml;base64,sign123")
                .is_ok()
        );
        assert!(visit.completed);
        assert_eq!(
            visit.customer_signature,
            Some("data:image/svg+xml;base64,sign123".into())
        );

        // Re-completing errors
        assert_eq!(
            visit.complete_visit("second_attempt"),
            Err(WarrantyError::VisitAlreadyCompleted {
                visit_id: "MV-2026-101".into()
            })
        );
    }
}
