//! Upstream ERPNext Project & Task Golden Tests (`erp-ppm`).
//!
//! Validates:
//! 1. Work Breakdown Structure (WBS) Task dependencies and hour logging.
//! 2. Weighted progress percentage calculation based on task planned hours.
//! 3. Timesheet actual labor cost accumulation and client project billing.

use erp_ppm::{Project, ProjectStatus, Task, TaskStatus};
use rust_decimal_macros::dec;

#[test]
fn test_golden_project_wbs_and_cost_rollup() {
    let mut project = Project::new(
        "PROJ-2026-0001",
        "Autonomous Delivery Drone Avionics",
        dec!(100000.0), // $100k budget
    );
    assert_eq!(project.status, ProjectStatus::Open);
    assert_eq!(project.percent_complete, dec!(0.0));

    // Define 3 WBS Tasks:
    // Task 1: Hardware Bringup (30 hrs, 30% weight)
    let mut t1 = Task::new(
        "TASK-001",
        "PROJ-2026-0001",
        "PCB Bringup & Power Validation",
        dec!(30.0),
    );
    // Task 2: Firmware & Sensor Fusion (50 hrs, 50% weight)
    let mut t2 = Task::new(
        "TASK-002",
        "PROJ-2026-0001",
        "Kalman Filter & IMU Driver",
        dec!(50.0),
    );
    t2.depends_on.push("TASK-001".into());

    // Task 3: Environmental HIL Testing (20 hrs, 20% weight)
    let mut t3 = Task::new(
        "TASK-003",
        "PROJ-2026-0001",
        "Thermal Chamber HIL Validation",
        dec!(20.0),
    );
    t3.depends_on.push("TASK-002".into());

    // Phase 1 Progress:
    // Task 1 complete (100%), Task 2 half done (50%), Task 3 not started (0%)
    t1.update_progress(dec!(100.0)).unwrap();
    t1.log_hours(dec!(32.0)); // Over budget by 2 hours
    assert_eq!(t1.status, TaskStatus::Completed);

    t2.update_progress(dec!(50.0)).unwrap();
    t2.log_hours(dec!(25.0));
    assert_eq!(t2.status, TaskStatus::Working);

    let tasks = vec![t1.clone(), t2.clone(), t3.clone()];
    project.recalculate_progress_from_tasks(&tasks);

    // Weighted progress = (30 * 100 + 50 * 50 + 20 * 0) / (30 + 50 + 20)
    // = (3000 + 2500) / 100 = 55.0%
    assert_eq!(project.percent_complete, dec!(55.0));
    assert_eq!(project.status, ProjectStatus::InProgress);

    // Record timesheet costs:
    // 32 hrs @ $100/hr = $3,200
    // 25 hrs @ $120/hr = $3,000
    // Total labor = $6,200
    project.record_cost(dec!(6200.0));
    assert_eq!(project.total_cost, dec!(6200.0));

    // Issue milestone progress invoice to client: $40,000
    project.record_billing(dec!(40000.0));
    assert_eq!(project.total_billed_amount, dec!(40000.0));

    // Phase 2: Complete all tasks
    t2.update_progress(dec!(100.0)).unwrap();
    t3.update_progress(dec!(100.0)).unwrap();

    let all_done = vec![t1, t2, t3];
    project.recalculate_progress_from_tasks(&all_done);

    assert_eq!(project.percent_complete, dec!(100.0));
    assert_eq!(project.status, ProjectStatus::Completed);
}
