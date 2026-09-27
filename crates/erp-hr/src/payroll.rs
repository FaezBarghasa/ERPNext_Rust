use crate::attendance::HrError;
use chrono::NaiveDate;
use erp_accounting::{JournalEntry, JournalEntryLine};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;
use tokio::task::JoinSet;

/// Employee salary structure configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SalaryStructure {
    /// Monthly base salary.
    pub base_salary: Decimal,
    /// House Rent Allowance percentage of basic (e.g. 20.0 for 20%).
    pub hra_percentage: Decimal,
    /// Standard statutory deduction.
    pub standard_deduction: Decimal,
    /// Income tax withholding percentage (e.g. 10.0 for 10%).
    pub tax_withholding_rate: Decimal,
}

/// Generated Employee Salary Slip detailing all earnings and deductions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SalarySlip {
    /// Slip document ID.
    pub name: String,
    /// Employee identifier.
    pub employee_id: String,
    /// Posting date / pay period end.
    pub posting_date: NaiveDate,
    /// Attended working days.
    pub attended_days: Decimal,
    /// Total standard working days in month.
    pub total_working_days: Decimal,
    /// Earned basic salary.
    pub earned_basic: Decimal,
    /// Earned HRA.
    pub earned_hra: Decimal,
    /// Overtime earnings.
    pub overtime_amount: Decimal,
    /// Gross earnings.
    pub gross_salary: Decimal,
    /// Income tax deducted.
    pub tax_deducted: Decimal,
    /// Other deductions (e.g. standard deduction, provident fund).
    pub other_deductions: Decimal,
    /// Total deductions.
    pub total_deductions: Decimal,
    /// Net take-home pay.
    pub net_salary: Decimal,
}

/// Salary Calculation Engine (Milestone 3.6).
pub struct SalaryCalculator;

impl SalaryCalculator {
    /// Computes individual salary slip based on attendance and salary structure.
    pub fn calculate_slip(
        employee_id: &str,
        structure: &SalaryStructure,
        attended_days: Decimal,
        total_working_days: Decimal,
        overtime_hours: Decimal,
        posting_date: NaiveDate,
        slip_id: &str,
    ) -> Result<SalarySlip, HrError> {
        if total_working_days <= Decimal::ZERO {
            return Err(HrError::SalaryCalculationError(
                employee_id.to_string(),
                "Total working days must be positive".to_string(),
            ));
        }

        let attendance_ratio = (attended_days / total_working_days).min(Decimal::ONE);
        let earned_basic = structure.base_salary * attendance_ratio;
        let earned_hra = earned_basic * (structure.hra_percentage / Decimal::from(100));

        // Overtime rate: 1.5x hourly rate (8-hour day)
        let hourly_rate = (structure.base_salary / total_working_days) / Decimal::from(8);
        let overtime_amount = overtime_hours * hourly_rate * Decimal::from_str_exact("1.5").unwrap();

        let gross_salary = earned_basic + earned_hra + overtime_amount;
        let tax_deducted = gross_salary * (structure.tax_withholding_rate / Decimal::from(100));
        let other_deductions = structure.standard_deduction;
        let total_deductions = tax_deducted + other_deductions;
        let net_salary = gross_salary - total_deductions;

        Ok(SalarySlip {
            name: slip_id.to_string(),
            employee_id: employee_id.to_string(),
            posting_date,
            attended_days,
            total_working_days,
            earned_basic,
            earned_hra,
            overtime_amount,
            gross_salary,
            tax_deducted,
            other_deductions,
            total_deductions,
            net_salary,
        })
    }
}

/// Payroll Accounting Configuration mapping GL accounts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayrollGlAccounts {
    pub salary_expense_account: String,
    pub payroll_payable_account: String,
    pub taxes_withheld_account: String,
    pub company: String,
}

/// Concurrent Enterprise Payroll Coordinator (Milestone 3.7).
pub struct EnterprisePayrollCoordinator;

impl EnterprisePayrollCoordinator {
    /// Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.
    pub async fn process_batch_payroll(
        employees: Vec<(String, SalaryStructure, Decimal, Decimal, Decimal)>,
        posting_date: NaiveDate,
    ) -> (Vec<SalarySlip>, Vec<(String, String)>) {
        let mut join_set = JoinSet::new();

        for (emp_id, struct_cfg, attended, total_days, ot_hours) in employees {
            join_set.spawn(async move {
                let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    SalaryCalculator::calculate_slip(
                        &emp_id,
                        &struct_cfg,
                        attended,
                        total_days,
                        ot_hours,
                        posting_date,
                        &format!("SLIP-{emp_id}"),
                    )
                }));

                match res {
                    Ok(Ok(slip)) => Ok(slip),
                    Ok(Err(e)) => Err((emp_id, e.to_string())),
                    Err(_) => Err((emp_id, "Thread panic in calculation".to_string())),
                }
            });
        }

        let mut successful_slips = Vec::new();
        let mut failed_records = Vec::new();

        while let Some(res) = join_set.join_next().await {
            if let Ok(outcome) = res {
                match outcome {
                    Ok(slip) => successful_slips.push(slip),
                    Err(failure) => failed_records.push(failure),
                }
            }
        }

        (successful_slips, failed_records)
    }

    /// Generates batch General Ledger journal entry for submitted payroll.
    /// - Debit: Salary Expense (Gross)
    /// - Credit: Payroll Payable (Net)
    /// - Credit: Taxes Withheld Payable (Tax)
    #[must_use]
    pub fn create_payroll_journal_entry(
        slips: &[SalarySlip],
        gl_config: &PayrollGlAccounts,
        posting_date: NaiveDate,
    ) -> JournalEntry {
        let total_gross: Decimal = slips.iter().map(|s| s.gross_salary).sum();
        let total_net: Decimal = slips.iter().map(|s| s.net_salary).sum();
        let total_tax: Decimal = slips.iter().map(|s| s.tax_deducted + s.other_deductions).sum();

        let lines = vec![
            JournalEntryLine {
                account: gl_config.salary_expense_account.clone(),
                debit: total_gross,
                credit: Decimal::ZERO,
                debit_in_account_currency: total_gross,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: gl_config.payroll_payable_account.clone(),
                debit: Decimal::ZERO,
                credit: total_net,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: total_net,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: gl_config.taxes_withheld_account.clone(),
                debit: Decimal::ZERO,
                credit: total_tax,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: total_tax,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
        ];

        JournalEntry {
            posting_date,
            company: gl_config.company.clone(),
            lines,
            remarks: format!("Batch Payroll for {} employees", slips.len()),
        }
    }
}

/// Leave Application record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaveApplication {
    pub employee_id: String,
    pub leave_type: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub total_days: Decimal,
}

/// Leave Allocation & Accrual Engine (Milestone 3.8).
pub struct LeaveEngine;

impl LeaveEngine {
    /// Validates leave application against remaining balance and existing leaves.
    pub fn validate_and_deduct_leave(
        application: &LeaveApplication,
        available_balance: Decimal,
        existing_leaves: &[LeaveApplication],
    ) -> Result<Decimal, HrError> {
        if application.total_days > available_balance {
            return Err(HrError::InsufficientLeaveBalance {
                leave_type: application.leave_type.clone(),
                requested: application.total_days,
                available: available_balance,
            });
        }

        for existing in existing_leaves {
            if existing.employee_id == application.employee_id {
                // Check overlap: max(from1, from2) <= min(to1, to2)
                if application.from_date.max(existing.from_date)
                    <= application.to_date.min(existing.to_date)
                {
                    return Err(HrError::OverlappingLeave {
                        from_date: existing.from_date,
                        to_date: existing.to_date,
                    });
                }
            }
        }

        Ok(available_balance - application.total_days)
    }
}
