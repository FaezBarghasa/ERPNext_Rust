---
okf_version: "0.2"
type: Class
title: LeaveApplication
description: Leave Application record.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:27:26Z"
concept_id: crates/erp-hr/src/payroll/LeaveApplication
language: rust
---

# LeaveApplication

Leave Application record.

## Signature

```rust
pub struct LeaveApplication
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Leave Application record.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `employee_id`
- `leave_type`
- `from_date`
- `to_date`
- `total_days`

## Source
Lines 221–227 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
