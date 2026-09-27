---
okf_version: "0.2"
type: Class
title: SalarySlip
description: Generated Employee Salary Slip detailing all earnings and deductions.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:31:25Z"
concept_id: crates/erp-hr/src/payroll/SalarySlip
language: rust
---

# SalarySlip

Generated Employee Salary Slip detailing all earnings and deductions.

## Signature

```rust
pub struct SalarySlip
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Generated Employee Salary Slip detailing all earnings and deductions.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `employee_id`
- `posting_date`
- `attended_days`
- `total_working_days`
- `earned_basic`
- `earned_hra`
- `overtime_amount`
- `gross_salary`
- `tax_deducted`
- `other_deductions`
- `total_deductions`
- `net_salary`

## Source
Lines 24–51 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
