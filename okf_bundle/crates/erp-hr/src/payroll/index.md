# payroll

## Classs

- [EnterprisePayrollCoordinator](EnterprisePayrollCoordinator.md) — Concurrent Enterprise Payroll Coordinator (Milestone 3.7).
- [LeaveApplication](LeaveApplication.md) — Leave Application record.
- [LeaveEngine](LeaveEngine.md) — Leave Allocation & Accrual Engine (Milestone 3.8).
- [PayrollGlAccounts](PayrollGlAccounts.md) — Payroll Accounting Configuration mapping GL accounts.
- [SalaryCalculator](SalaryCalculator.md) — Salary Calculation Engine (Milestone 3.6).
- [SalarySlip](SalarySlip.md) — Generated Employee Salary Slip detailing all earnings and deductions.
- [SalaryStructure](SalaryStructure.md) — Employee salary structure configuration.

## Functions

- [calculate_slip](calculate_slip.md) — Computes individual salary slip based on attendance and salary structure.
- [calculate_slip](calculate_slip_1.md) — Computes individual salary slip based on attendance and salary structure.
- [create_payroll_journal_entry](create_payroll_journal_entry.md) — Generates batch General Ledger journal entry for submitted payroll.
- [create_payroll_journal_entry](create_payroll_journal_entry_1.md) — Generates batch General Ledger journal entry for submitted payroll.
- [process_batch_payroll](process_batch_payroll.md) — Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.
- [process_batch_payroll](process_batch_payroll_1.md) — Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.
- [validate_and_deduct_leave](validate_and_deduct_leave.md) — Validates leave application against remaining balance and existing leaves.
- [validate_and_deduct_leave](validate_and_deduct_leave_1.md) — Validates leave application against remaining balance and existing leaves.
