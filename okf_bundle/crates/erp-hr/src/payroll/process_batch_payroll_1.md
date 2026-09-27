---
okf_version: "0.2"
type: Function
title: process_batch_payroll
description: Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:31:25Z"
concept_id: crates/erp-hr/src/payroll/process_batch_payroll_1
language: rust
---

# process_batch_payroll

Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.

## Signature

```rust
pub fn process_batch_payroll(
        employees: Vec<(String, SalaryStructure, Decimal, Decimal, Decimal)>,
        posting_date: NaiveDate,
    ) -> (Vec<SalarySlip>, Vec<(String, String)>)
```

## Visibility

- `pub`

## Docstring

Runs concurrent payroll calculation across workforce using Tokio JoinSet with error isolation.

## Source
Lines 120–161 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
