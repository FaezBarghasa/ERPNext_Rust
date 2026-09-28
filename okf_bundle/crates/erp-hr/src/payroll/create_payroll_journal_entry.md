---
okf_version: "0.2"
type: Function
title: create_payroll_journal_entry
description: Generates batch General Ledger journal entry for submitted payroll.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-hr/src/payroll/create_payroll_journal_entry
language: rust
---

# create_payroll_journal_entry

Generates batch General Ledger journal entry for submitted payroll.

## Signature

```rust
impl EnterprisePayrollCoordinator { pub fn create_payroll_journal_entry(
        slips: &[SalarySlip],
        gl_config: &PayrollGlAccounts,
        posting_date: NaiveDate,
    ) -> JournalEntry }
```

## Visibility

- `pub`

## Docstring

Generates batch General Ledger journal entry for submitted payroll.
- Debit: Salary Expense (Gross)
- Credit: Payroll Payable (Net)
- Credit: Taxes Withheld Payable (Tax)
[must_use]

## Source
Lines 169–220 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
