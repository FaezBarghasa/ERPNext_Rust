//! Upstream ERPNext HR & Payroll Golden Tests (`erp-hr`).
//!
//! Validates:
//! 1. Employee Master onboarding, Department, and Designation assignment.
//! 2. Biometric attendance punch logging and shift reconciliation (Present, LateEntry).
//! 3. Leave Application submission, approval, and balance validation.
//! 4. Overtime calculation with standard and holiday multipliers.
//! 5. Monthly Salary Slip generation: Gross Earnings, Deductions, and Net Payable.
//! 6. Payroll Accounting General Ledger journal synchronization and balanced entry validation.
//! 7. Employee separation and relieving validation.

use chrono::{NaiveDate, NaiveTime};
use erp_hr::{
    attendance::AttendanceReconciler, attendance::AttendanceStatus, attendance::ShiftType,
    employee::Department, employee::Designation, employee::Employee, employee::EmployeeStatus,
    employee::EmploymentType, overtime::OvertimeCalculationRequest, overtime::OvertimeEngine,
    overtime::OvertimeMultiplier, payroll::EnterprisePayrollCoordinator, payroll::LeaveApplication,
    payroll::LeaveEngine, payroll::PayrollGlAccounts, payroll::SalaryCalculator,
    payroll::SalaryStructure,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_golden_hr_employee_attendance_payroll_and_gl_flow() {
    let join_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // 1. Employee Onboarding & Department / Designation
    let dept = Department::new("DEP-001", "Robotics Systems R&D", None::<&str>);
    assert_eq!(dept.department_name, "Robotics Systems R&D");

    let designation = Designation::new("DES-001", "Lead Embedded Engineer");
    assert_eq!(designation.designation_name, "Lead Embedded Engineer");

    let mut employee = Employee::new(
        "HR-EMP-2026-0001",
        "Jordan",
        "Miller",
        "Robotics Systems R&D",
        "Lead Embedded Engineer",
        join_date,
        EmploymentType::FullTime,
    );
    employee.salary_structure = Some("STRUCT-ENG-L3".into());
    assert_eq!(employee.status, EmployeeStatus::Active);

    // 2. Shift Reconciliation (9:00 AM - 5:00 PM with 15 min grace)
    let shift = ShiftType {
        name: "Standard Core Hours".into(),
        start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        late_entry_grace_mins: 15,
        early_exit_grace_mins: 15,
    };

    // Day 1: In at 9:08, out at 17:05 -> Present
    let status_day1 = AttendanceReconciler::reconcile_shift(
        &shift,
        Some(NaiveTime::from_hms_opt(9, 8, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(17, 5, 0).unwrap()),
    );
    assert_eq!(status_day1, AttendanceStatus::Present);

    // Day 2: In at 9:45, out at 17:00 -> LateEntry
    let status_day2 = AttendanceReconciler::reconcile_shift(
        &shift,
        Some(NaiveTime::from_hms_opt(9, 45, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
    );
    assert_eq!(status_day2, AttendanceStatus::LateEntry);

    // 3. Leave Application (2 days Paid Vacation)
    let leave_app = LeaveApplication {
        employee_id: "HR-EMP-2026-0001".into(),
        leave_type: "Casual Leave".into(),
        from_date: NaiveDate::from_ymd_opt(2026, 10, 8).unwrap(),
        to_date: NaiveDate::from_ymd_opt(2026, 10, 9).unwrap(),
        total_days: dec!(2.0),
    };
    let remaining_leave =
        LeaveEngine::validate_and_deduct_leave(&leave_app, dec!(14.0), &[]).unwrap();
    assert_eq!(remaining_leave, dec!(12.0));

    // 4. Overtime Slip (2 hours on weekday @ 1.5x)
    let shift_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
    let actual_out = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
    let ot_req = OvertimeCalculationRequest {
        slip_id: "OT-2026-10-001",
        employee_id: "HR-EMP-2026-0001",
        attendance_date: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
        shift_end_time: shift_end,
        actual_out_time: actual_out,
        base_hourly_rate: dec!(50.0), // $50/hr
        multiplier: OvertimeMultiplier::Standard1Point5,
        min_overtime_mins_threshold: 30,
    };
    let ot_slip = OvertimeEngine::generate_overtime_slip(&ot_req).expect("Valid overtime slip");
    // 2 hours * $50 * 1.5 = $150.00
    assert_eq!(ot_slip.overtime_hours, dec!(2.0));
    assert_eq!(ot_slip.overtime_amount, dec!(150.0));

    // 5. Salary Structure & Salary Slip Calculation
    // Base salary: $8,000, HRA: 20%, Standard deduction: $500, Tax withholding: 10%
    let structure = SalaryStructure {
        base_salary: dec!(8000.0),
        hra_percentage: dec!(20.0),
        standard_deduction: dec!(500.0),
        tax_withholding_rate: dec!(10.0),
    };

    // 20 attended days out of 20 total working days, 2 overtime hours
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
    let slip = SalaryCalculator::calculate_slip(
        "HR-EMP-2026-0001",
        &structure,
        dec!(20.0),
        dec!(20.0),
        dec!(2.0),
        posting_date,
        "SLIP-2026-10-001",
    )
    .expect("Valid salary slip");

    // Earned basic = $8,000 * 1.0 = $8,000
    assert_eq!(slip.earned_basic, dec!(8000.0));
    // Earned HRA = 20% of 8,000 = $1,600
    assert_eq!(slip.earned_hra, dec!(1600.0));
    // Hourly rate = 8000 / 20 / 8 = $50/hr. OT amount = 2 * 50 * 1.5 = $150.00
    assert_eq!(slip.overtime_amount, dec!(150.0));
    // Gross salary = 8000 + 1600 + 150 = $9,750
    assert_eq!(slip.gross_salary, dec!(9750.0));
    // Tax deducted = 10% of 9750 = $975
    assert_eq!(slip.tax_deducted, dec!(975.0));
    // Other deductions = $500
    assert_eq!(slip.other_deductions, dec!(500.0));
    // Total deductions = 975 + 500 = $1,475
    assert_eq!(slip.total_deductions, dec!(1475.0));
    // Net salary = 9750 - 1475 = $8,275
    assert_eq!(slip.net_salary, dec!(8275.0));

    // 6. Payroll Accounting General Ledger Journal Entry Synchronization
    let gl_config = PayrollGlAccounts {
        salary_expense_account: "5100 - Salary Expense".into(),
        payroll_payable_account: "2110 - Payroll Payable".into(),
        taxes_withheld_account: "2120 - Taxes Withheld Payable".into(),
        company: "Acme Engineering Corp".into(),
    };

    let journal = EnterprisePayrollCoordinator::create_payroll_journal_entry(
        &[slip],
        &gl_config,
        posting_date,
    );

    // Verify debit == credit and journal validation
    assert!(journal.validate_balance().is_ok());
    let total_debit: Decimal = journal.lines.iter().map(|l| l.debit).sum();
    let total_credit: Decimal = journal.lines.iter().map(|l| l.credit).sum();
    assert_eq!(total_debit, total_credit);
    assert_eq!(total_debit, dec!(9750.0));

    // 7. Employee Separation
    let separate_date = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    employee.separate(separate_date).unwrap();
    assert_eq!(employee.status, EmployeeStatus::Left);
    assert_eq!(employee.relieving_date, Some(separate_date));
}
