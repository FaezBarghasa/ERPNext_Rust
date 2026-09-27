---
okf_version: "0.2"
type: Function
title: calculate_slip
description: Computes individual salary slip based on attendance and salary structure.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:31:25Z"
concept_id: crates/erp-hr/src/payroll/calculate_slip
language: rust
---

# calculate_slip

Computes individual salary slip based on attendance and salary structure.

## Signature

```rust
impl SalaryCalculator { pub fn calculate_slip(
        employee_id: &str,
        structure: &SalaryStructure,
        attended_days: Decimal,
        total_working_days: Decimal,
        overtime_hours: Decimal,
        posting_date: NaiveDate,
        slip_id: &str,
    ) -> Result<SalarySlip, HrError> }
```

## Visibility

- `pub`

## Docstring

Computes individual salary slip based on attendance and salary structure.

## Source
Lines 58–103 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
