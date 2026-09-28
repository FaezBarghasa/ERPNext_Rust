---
okf_version: "0.2"
type: Function
title: reconcile_shift
description: Reconciles daily check-in and check-out against shift boundaries.
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-hr/src/attendance/reconcile_shift_1
language: rust
---

# reconcile_shift

Reconciles daily check-in and check-out against shift boundaries.

## Signature

```rust
pub fn reconcile_shift(
        shift: &ShiftType,
        check_in: Option<NaiveTime>,
        check_out: Option<NaiveTime>,
    ) -> AttendanceStatus
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Reconciles daily check-in and check-out against shift boundaries.
[must_use]

## Source
Lines 97–129 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
