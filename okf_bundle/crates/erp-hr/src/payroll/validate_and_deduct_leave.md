---
okf_version: "0.2"
type: Function
title: validate_and_deduct_leave
description: Validates leave application against remaining balance and existing leaves.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:31:25Z"
concept_id: crates/erp-hr/src/payroll/validate_and_deduct_leave
language: rust
---

# validate_and_deduct_leave

Validates leave application against remaining balance and existing leaves.

## Signature

```rust
impl LeaveEngine { pub fn validate_and_deduct_leave(
        application: &LeaveApplication,
        available_balance: Decimal,
        existing_leaves: &[LeaveApplication],
    ) -> Result<Decimal, HrError> }
```

## Visibility

- `pub`

## Docstring

Validates leave application against remaining balance and existing leaves.

## Source
Lines 234–262 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
