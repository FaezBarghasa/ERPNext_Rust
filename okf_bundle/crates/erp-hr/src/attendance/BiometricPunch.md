---
okf_version: "0.2"
type: Class
title: BiometricPunch
description: Raw biometric timestamp check-in / check-out punch log.
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-hr/src/attendance/BiometricPunch
language: rust
---

# BiometricPunch

Raw biometric timestamp check-in / check-out punch log.

## Signature

```rust
pub struct BiometricPunch
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Raw biometric timestamp check-in / check-out punch log.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `employee_id`
- `device_id`
- `timestamp_date`
- `punch_time`

## Source
Lines 34–43 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
