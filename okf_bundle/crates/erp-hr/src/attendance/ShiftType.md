---
okf_version: "0.2"
type: Class
title: ShiftType
description: Shift specification with grace period margins.
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:30:45Z"
concept_id: crates/erp-hr/src/attendance/ShiftType
language: rust
---

# ShiftType

Shift specification with grace period margins.

## Signature

```rust
pub struct ShiftType
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Shift specification with grace period margins.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `start_time`
- `end_time`
- `late_entry_grace_mins`
- `early_exit_grace_mins`

## Source
Lines 66–77 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
