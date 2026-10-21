//! `erp-quality` — Enterprise Quality Control, NCR & CAPA Engine for `rust-web-land (rwl)`.
//!
//! Provides comprehensive ISO 9001 quality management:
//! - Inspection: `QualityInspection`, `QualityInspectionTemplate`, `QualityReading`
//! - Non-Conformance: `NonConformanceReport`, `NcrSeverity`, `NcrDisposition`
//! - Remediation: `CapaRecord`, `CapaActionItem`, `CapaActionType`

pub mod capa;
pub mod inspection;
pub mod ncr;

pub use capa::{CapaActionItem, CapaActionType, CapaRecord, CapaStatus};
pub use inspection::{
    InspectionParameterDef, InspectionStatus, InspectionType, QualityError, QualityInspection,
    QualityInspectionTemplate, QualityReading,
};
pub use ncr::{NcrDisposition, NcrSeverity, NcrStatus, NonConformanceReport};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_quality_inspection_pass_flow() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let mut qi = QualityInspection::new(
            "QI-2026-00001",
            InspectionType::Incoming,
            "BEARING-608",
            "Purchase Receipt",
            "PREC-2026-00100",
            5,
            date,
            "inspector@enterprise.local",
        )
        .expect("Failed to initialize QI");

        // Add 2 passing readings
        qi.add_reading(QualityReading::evaluate(
            "Outer Diameter",
            dec!(21.95),
            dec!(22.05),
            dec!(22.00),
        ))
        .unwrap();
        qi.add_reading(QualityReading::evaluate(
            "Hardness HRC",
            dec!(58.0),
            dec!(64.0),
            dec!(61.5),
        ))
        .unwrap();

        let verdict = qi.submit().expect("Submission failed");
        assert_eq!(verdict, InspectionStatus::Accepted);
        assert_eq!(qi.status, InspectionStatus::Accepted);
    }

    #[test]
    fn test_quality_inspection_fail_flow() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let mut qi = QualityInspection::new(
            "QI-2026-00002",
            InspectionType::InProcess,
            "SHAFT-M10",
            "Job Card",
            "JC-2026-00045",
            10,
            date,
            "inspector@enterprise.local",
        )
        .expect("Failed to initialize QI");

        // One pass, one fail reading (9.80 < 9.95)
        qi.add_reading(QualityReading::evaluate(
            "Length",
            dec!(100.0),
            dec!(102.0),
            dec!(101.0),
        ))
        .unwrap();
        qi.add_reading(QualityReading::evaluate(
            "Diameter",
            dec!(9.95),
            dec!(10.05),
            dec!(9.80),
        ))
        .unwrap();

        let verdict = qi.submit().expect("Submission failed");
        assert_eq!(verdict, InspectionStatus::Rejected);
        assert_eq!(qi.status, InspectionStatus::Rejected);
    }

    #[test]
    fn test_ncr_and_capa_end_to_end_lifecycle() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();

        // 1. Raise NCR from rejected inspection
        let mut ncr = NonConformanceReport::new(
            "NCR-2026-0001",
            "SHAFT-M10",
            dec!(25.0),
            date,
            NcrSeverity::Major,
            "Undersized diameter on CNC lathe turning operation",
            "Quarantine batch in scrap cage",
        );
        ncr.inspection_ref = Some("QI-2026-00002".into());
        assert_eq!(ncr.status, NcrStatus::Open);

        // 2. Decide disposition: Rework
        ncr.decide_disposition(
            NcrDisposition::Rework,
            "Tool wear offset drift on Machine #4",
        );
        assert_eq!(ncr.status, NcrStatus::DispositionDecided);

        // 3. Initiate CAPA
        let mut capa = CapaRecord::new(
            "CAPA-2026-0001",
            Some("NCR-2026-0001"),
            CapaActionType::Corrective,
            "Recurring undersized shaft turning",
            "Automated tool touch probe not recalibrated after tip replacement",
        );
        ncr.link_capa("CAPA-2026-0001");
        assert_eq!(ncr.status, NcrStatus::CapaRequired);

        // 4. Add action items and execute
        capa.add_action_item(CapaActionItem::new(
            1,
            "Recalibrate probe sensor and update CNC macro offsets",
            "maintenance_tech",
            date,
        ));
        capa.approve_plan();
        assert_eq!(capa.status, CapaStatus::InImplementation);

        capa.action_items[0].complete(date);
        assert!(capa.check_implementation_complete());
        assert_eq!(capa.status, CapaStatus::PendingEffectivenessReview);

        // 5. Conduct effectiveness review and close
        let review_date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        capa.close_with_effectiveness_review(
            "Zero dimension deviations recorded across subsequent 500 parts",
            review_date,
        );
        assert_eq!(capa.status, CapaStatus::Closed);

        ncr.close();
        assert_eq!(ncr.status, NcrStatus::Closed);
    }
}
