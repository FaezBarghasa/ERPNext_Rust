---
okf_version: "0.2"
type: Class
title: PayrollGlAccounts
description: Payroll Accounting Configuration mapping GL accounts.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-hr/src/payroll/PayrollGlAccounts
language: rust
---

# PayrollGlAccounts

Payroll Accounting Configuration mapping GL accounts.

## Signature

```rust
pub struct PayrollGlAccounts
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Payroll Accounting Configuration mapping GL accounts.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `salary_expense_account`
- `payroll_payable_account`
- `taxes_withheld_account`
- `company`

## Source
Lines 109–114 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
