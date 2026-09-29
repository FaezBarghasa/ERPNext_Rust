pub mod advances;
pub mod arrears;
pub mod attendance;
pub mod benefits;
pub mod dynamic_compensation;
pub mod overtime;
pub mod payroll;

pub use advances::{EmployeeAdvance, ExpenseClaimDetail, ExpenseClaimEngine, SettledExpenseClaim};
pub use arrears::{ArrearsEngine, RetroactiveArrearsSummary, RetroactiveMonthDifference};
pub use attendance::{
    AttendanceReconciler, AttendanceStatus, BiometricIngestionGateway, BiometricPunch, HrError,
    ShiftType,
};
pub use benefits::{FlexBenefitCategory, FlexBenefitLedger, HolidayList};
pub use dynamic_compensation::{
    CompensationModelType, DynamicCompensationEngine, DynamicCompensationResult,
    JobCardPieceworkLog, MilestoneBonusLog, ShiftHoursBreakdown,
};
pub use overtime::{OvertimeEngine, OvertimeMultiplier, OvertimeSlip};
pub use payroll::{
    EnterprisePayrollCoordinator, LeaveApplication, LeaveEngine, PayrollGlAccounts,
    SalaryCalculator, SalarySlip, SalaryStructure,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};
    use rust_decimal_macros::dec;

    #[test]
    fn test_shift_reconciliation() {
        let shift = ShiftType {
            name: "Standard 9-5".into(),
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            late_entry_grace_mins: 15,
            early_exit_grace_mins: 15,
        };

        // On time: 9:05 in, 17:00 out -> Present
        let status1 = AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 5, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
        );
        assert_eq!(status1, AttendanceStatus::Present);

        // Late: 9:30 in, 17:00 out -> LateEntry
        let status2 = AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
        );
        assert_eq!(status2, AttendanceStatus::LateEntry);

        // Early Exit: 9:00 in, 16:30 out -> EarlyExit
        let status3 = AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(16, 30, 0).unwrap()),
        );
        assert_eq!(status3, AttendanceStatus::EarlyExit);

        // Half day: worked only 3 hours (9:00 to 12:00) -> HalfDay
        let status4 = AttendanceReconciler::reconcile_shift(
            &shift,
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
        );
        assert_eq!(status4, AttendanceStatus::HalfDay);

        // Absent: No punch -> Absent
        let status5 = AttendanceReconciler::reconcile_shift(&shift, None, None);
        assert_eq!(status5, AttendanceStatus::Absent);
    }

    #[tokio::test]
    async fn test_salary_calculation_and_concurrent_payroll() {
        let structure = SalaryStructure {
            base_salary: dec!(4000.00),
            hra_percentage: dec!(20.0), // 20% of basic
            standard_deduction: dec!(100.00),
            tax_withholding_rate: dec!(10.0), // 10% of gross
        };

        let date = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();

        // 15 attended days out of 20 -> Basic = 3000.00, HRA = 600.00 -> Gross = 3600.00
        // Tax (10%) = 360.00 + Std Ded = 100.00 -> Total Ded = 460.00 -> Net = 3140.00
        let slip = SalaryCalculator::calculate_slip(
            "EMP-001",
            &structure,
            dec!(15.0),
            dec!(20.0),
            dec!(0.0),
            date,
            "SLIP-001",
        )
        .expect("Salary calculation failed");

        assert_eq!(slip.earned_basic, dec!(3000.00));
        assert_eq!(slip.earned_hra, dec!(600.00));
        assert_eq!(slip.gross_salary, dec!(3600.00));
        assert_eq!(slip.tax_deducted, dec!(360.00));
        assert_eq!(slip.net_salary, dec!(3140.00));

        // Concurrent payroll batch test across multiple employees
        let employees = vec![
            (
                "EMP-001".into(),
                structure.clone(),
                dec!(20.0),
                dec!(20.0),
                dec!(0.0),
            ),
            (
                "EMP-002".into(),
                structure.clone(),
                dec!(18.0),
                dec!(20.0),
                dec!(5.0),
            ),
            (
                "EMP-003".into(),
                structure.clone(),
                dec!(20.0),
                dec!(20.0),
                dec!(0.0),
            ),
        ];

        let (slips, failed) =
            EnterprisePayrollCoordinator::process_batch_payroll(employees, date).await;
        assert_eq!(failed.len(), 0);
        assert_eq!(slips.len(), 3);

        // General Ledger Journal Entry verification
        let gl_config = PayrollGlAccounts {
            salary_expense_account: "5110 - Salary Expense".into(),
            payroll_payable_account: "2110 - Payroll Payable".into(),
            taxes_withheld_account: "2230 - Taxes Withheld Payable".into(),
            company: "Acme Corp".into(),
        };

        let jv =
            EnterprisePayrollCoordinator::create_payroll_journal_entry(&slips, &gl_config, date);
        assert!(jv.validate_balance().is_ok());
    }

    #[test]
    fn test_leave_allocation_and_overlap_detection() {
        let app1 = LeaveApplication {
            employee_id: "EMP-001".into(),
            leave_type: "Annual Leave".into(),
            from_date: NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
            to_date: NaiveDate::from_ymd_opt(2026, 10, 5).unwrap(),
            total_days: dec!(5.0),
        };

        // Available balance: 10 days -> Deduct 5 days -> Remaining: 5 days
        let rem1 = LeaveEngine::validate_and_deduct_leave(&app1, dec!(10.0), &[])
            .expect("Leave validation failed");
        assert_eq!(rem1, dec!(5.0));

        // Attempt requesting 6 days with only 5 available -> Insufficient balance error
        let app2 = LeaveApplication {
            employee_id: "EMP-001".into(),
            leave_type: "Annual Leave".into(),
            from_date: NaiveDate::from_ymd_opt(2026, 10, 10).unwrap(),
            to_date: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
            total_days: dec!(6.0),
        };
        let err =
            LeaveEngine::validate_and_deduct_leave(&app2, dec!(5.0), std::slice::from_ref(&app1));
        assert!(matches!(err, Err(HrError::InsufficientLeaveBalance { .. })));

        // Attempt requesting overlapping leave (Oct 3 to Oct 7) -> Overlap error
        let app_overlap = LeaveApplication {
            employee_id: "EMP-001".into(),
            leave_type: "Casual Leave".into(),
            from_date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(),
            to_date: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
            total_days: dec!(5.0),
        };
        let overlap_err = LeaveEngine::validate_and_deduct_leave(&app_overlap, dec!(20.0), &[app1]);
        assert!(matches!(overlap_err, Err(HrError::OverlappingLeave { .. })));
    }
}
