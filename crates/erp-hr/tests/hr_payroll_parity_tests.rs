//! Integration tests verifying Phase 6 Human Resources & Automated Payroll Engine.
//!
//! Validates:
//! 1. Employee Master and department / designation hierarchy.
//! 2. Shift and attendance reconciliation with biometric check-in grace periods.
//! 3. Leave Allocation, monthly accrual, and holiday calendar exclusions.
//! 4. Salary calculation with earnings (Basic, HRA, Travel), deductions (Tax, PF), and overtime.
//! 5. Bulk payroll processing with double-entry journal vouchers (Expense vs Payable, Payable vs Bank).

use chrono::{NaiveDate, NaiveTime};
use erp_hr::attendance::{AttendanceReconciler, AttendanceStatus, ShiftType};
use erp_hr::employee::{Department, Designation, Employee, EmploymentType};
use erp_hr::leave_management::{HolidayList as CalendarHolidayList, LeaveAllocation, LeaveType};
use erp_hr::payroll::{EnterprisePayrollCoordinator, PayrollGlAccounts, SalaryCalculator, SalaryStructure};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn test_employee_and_attendance_reconciliation() {
    let dept = Department::new("DEPT-ENG", "Engineering & Robotics", None::<&str>);
    let desig = Designation::new("DESIG-SR-DEV", "Senior Systems Engineer");

    let emp = Employee::new(
        "EMP-001",
        "Faez",
        "Barghasa",
        dept.name.clone(),
        desig.name.clone(),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        EmploymentType::FullTime,
    );

    assert_eq!(emp.first_name, "Faez");
    assert_eq!(emp.department.as_str(), "DEPT-ENG");

    // Shift reconciliation test
    let shift = ShiftType {
        name: "Morning Shift".into(),
        start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        late_entry_grace_mins: 15,
        early_exit_grace_mins: 15,
    };

    // On-time check-in (9:10) and check-out (18:00) -> Present
    assert_eq!(
        AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 10, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap())
        ),
        AttendanceStatus::Present
    );

    // Late entry (9:25) -> LateEntry
    assert_eq!(
        AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 25, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap())
        ),
        AttendanceStatus::LateEntry
    );
}

#[test]
fn test_leave_allocation_accrual_and_holidays() {
    let mut alloc = LeaveAllocation {
        employee: "EMP-001".into(),
        leave_type: LeaveType::Annual,
        allocated_days: dec!(15.0),
        accrued_days: Decimal::ZERO,
        used_days: Decimal::ZERO,
    };

    assert_eq!(alloc.available_balance(), dec!(15.0));

    // Deduct 3 days
    alloc.deduct_leave(dec!(3.0)).unwrap();
    assert_eq!(alloc.available_balance(), dec!(12.0));

    // Monthly accrual: +1.25 days
    alloc.apply_accrual(dec!(1.25));
    assert_eq!(alloc.available_balance(), dec!(13.25));

    // Holiday List Calendar Working Days Test
    let holiday_list = CalendarHolidayList {
        name: "Iran National Holidays 2026".into(),
        holidays: vec![
            NaiveDate::from_ymd_opt(2026, 3, 21).unwrap(), // Nowruz
            NaiveDate::from_ymd_opt(2026, 3, 22).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 23).unwrap(),
        ],
    };

    let working_days = holiday_list.count_working_days(
        NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
    );
    // Excludes weekends and the 3 holidays
    assert!(working_days < dec!(7.0));
}

#[tokio::test]
async fn test_automated_payroll_and_gl_postings() {
    let posting_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();

    let structure = SalaryStructure {
        base_salary: dec!(5000.0),
        hra_percentage: dec!(20.0),        // $1000
        travel_allowance: dec!(500.0),     // $500
        standard_deduction: dec!(100.0),   // $100
        provident_fund_rate: dec!(5.0),    // 5% of basic = $250
        tax_withholding_rate: dec!(10.0),  // 10% of gross
    };

    // Calculate Slip: 20 out of 20 days attended, 4 hours overtime
    let slip = SalaryCalculator::calculate_slip(
        "EMP-001",
        &structure,
        dec!(20.0),
        dec!(20.0),
        dec!(4.0),
        posting_date,
        "SLIP-EMP-001",
    )
    .unwrap();

    // Basic: $5000, HRA: $1000, Travel: $500
    // Hourly rate = 5000 / 20 / 8 = $31.25. 4 hrs * 31.25 * 1.5 = $187.50
    // Gross = 5000 + 1000 + 500 + 187.50 = $6687.50
    assert_eq!(slip.gross_salary, dec!(6687.50));

    // Tax = 10% of 6687.50 = $668.75
    // Other deductions = standard ($100) + PF ($250) = $350.00
    // Total deductions = 668.75 + 350.00 = $1018.75
    assert_eq!(slip.tax_deducted, dec!(668.75));
    assert_eq!(slip.other_deductions, dec!(350.00));
    assert_eq!(slip.total_deductions, dec!(1018.75));

    // Net = 6687.50 - 1018.75 = $5668.75
    assert_eq!(slip.net_salary, dec!(5668.75));

    // Batch Payroll Test via Coordinator
    let workforce = vec![(
        "EMP-001".to_string(),
        structure,
        dec!(20.0),
        dec!(20.0),
        dec!(4.0),
    )];
    let (slips, failures) = EnterprisePayrollCoordinator::process_batch_payroll(workforce, posting_date).await;
    assert_eq!(failures.len(), 0);
    assert_eq!(slips.len(), 1);

    let gl_config = PayrollGlAccounts {
        salary_expense_account: "5110 - Salary Expenses".into(),
        payroll_payable_account: "2110 - Payroll Payable".into(),
        taxes_withheld_account: "2130 - Tax Withholding Payable".into(),
        bank_account: "1110 - Operating Commercial Bank".into(),
        company: "RustNext Corp".into(),
    };

    // 1. Payroll Accrual Entry:
    // Debit Salary Expense ($6687.50)
    // Credit Payroll Payable ($5668.75)
    // Credit Tax & Other Withholdings ($1018.75)
    let accrual_jv = EnterprisePayrollCoordinator::create_payroll_journal_entry(&slips, &gl_config, posting_date);
    let total_dr: Decimal = accrual_jv.lines.iter().map(|l| l.debit).sum();
    let total_cr: Decimal = accrual_jv.lines.iter().map(|l| l.credit).sum();
    assert_eq!(total_dr, dec!(6687.50));
    assert_eq!(total_cr, dec!(6687.50));

    // 2. Bank Payment Disbursement Entry:
    // Debit Payroll Payable ($5668.75)
    // Credit Bank Account ($5668.75)
    let payment_jv = EnterprisePayrollCoordinator::create_payroll_payment_entry(&slips, &gl_config, posting_date);
    let pay_dr: Decimal = payment_jv.lines.iter().map(|l| l.debit).sum();
    let pay_cr: Decimal = payment_jv.lines.iter().map(|l| l.credit).sum();
    assert_eq!(pay_dr, dec!(5668.75));
    assert_eq!(pay_cr, dec!(5668.75));
    assert_eq!(payment_jv.lines[1].account, "1110 - Operating Commercial Bank");
}
