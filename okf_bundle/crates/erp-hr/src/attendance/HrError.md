---
okf_version: "0.2"
type: Class
title: HrError
description: "HR & Attendance errors."
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:30:45Z"
concept_id: crates/erp-hr/src/attendance/HrError
language: rust
---

# HrError

HR & Attendance errors.

## Signature

```rust
pub enum HrError
```

## Decorators

- `derive(Debug, Error, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

HR & Attendance errors.
[derive(Debug, Error, PartialEq, Eq)]

## Methods

- `leave_type`
- `requested`
- `available`
- `from_date`
- `to_date`

## Source
Lines 8–28 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
