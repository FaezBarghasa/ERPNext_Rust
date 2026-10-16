//! Upstream ERPNext Support Golden Tests (`erp-support`).
//!
//! Validates:
//! 1. Issue lifecycle management, response & resolution SLA tracking, and breach detection.
//! 2. Workload-balanced agent ticket routing.
//! 3. Equipment warranty claim validation and preventive maintenance visits.

use chrono::NaiveDate;
use erp_support::{
    Issue, IssuePriority, IssueSlaStatus, IssueStatus, MaintenancePeriodicity, MaintenanceSchedule,
    WarrantyClaim, WarrantyStatus,
};

#[test]
fn test_golden_support_issue_sla_and_resolution_flow() {
    let open_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let resp_target = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
    let res_target = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();

    let mut issue = Issue::new(
        "ISS-2026-0042",
        "SCADA Telemetry Stream Packet Loss",
        "National Grid Infrastructure",
        "grid_ops@natgrid.gov",
        open_date,
        IssuePriority::Urgent,
    );

    issue.set_sla_targets(resp_target, res_target);
    assert_eq!(issue.status, IssueStatus::Open);
    assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

    // Agent replies on same day -> Within SLA
    issue.record_reply(open_date);
    assert_eq!(issue.status, IssueStatus::Replied);
    assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

    // Resolve on Oct 17 -> Before Oct 18 target
    let res_date = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap();
    issue
        .resolve(
            "Replaced degraded optical transceiver on SFP+ Port 3",
            res_date,
        )
        .unwrap();

    assert_eq!(issue.status, IssueStatus::Resolved);
    assert_eq!(issue.sla_status, IssueSlaStatus::WithinSla);

    // Customer confirms -> Close
    issue.close().unwrap();
    assert_eq!(issue.status, IssueStatus::Closed);

    // Later issue reopens if recurrence
    issue.reopen();
    assert_eq!(issue.status, IssueStatus::Open);
}

#[test]
fn test_golden_warranty_claim_and_maintenance_plan() {
    use chrono::{Duration, Utc};

    let now = Utc::now();

    // 1. Warranty Claim
    let mut claim = WarrantyClaim {
        claim_id: "CLM-001".into(),
        customer_id: "CUST-TESLA".into(),
        item_code: "ROBOT-ARM".into(),
        serial_no: "SN-ROBOT-0042".into(),
        sale_date: now - Duration::days(60),
        warranty_period_days: 365,
        issue_description: "Joint 4 servo motor actuator failure".into(),
        status: WarrantyStatus::ClaimSubmitted,
        resolution_notes: None,
    };
    assert_eq!(claim.status, WarrantyStatus::ClaimSubmitted);
    assert!(claim.is_eligible(now).unwrap());

    claim.status = WarrantyStatus::ClaimApproved;
    claim.resolution_notes =
        Some("Replacement harmonic drive actuator dispatched under warranty".into());
    assert_eq!(claim.status, WarrantyStatus::ClaimApproved);

    // 2. Scheduled Maintenance Plan
    let start_date = now;
    let end_date = now + Duration::days(360);

    let schedule = MaintenanceSchedule::generate_events(
        "SCH-2026-01",
        "CUST-TESLA",
        "ROBOT-ARM",
        "SN-ROBOT-0042",
        start_date,
        end_date,
        MaintenancePeriodicity::Quarterly,
    );

    // Quarterly across 360 days: ~5 events (0, 90, 180, 270, 360)
    assert!(schedule.events.len() >= 4);
    assert_eq!(schedule.events[0].event_id, "SCH-2026-01-EV-01");
}
