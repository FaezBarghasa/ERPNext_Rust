---
okf_version: "0.2"
type: Function
title: new
description: "Creates a bounded 10,000 capacity ingestion gateway."
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:30:45Z"
concept_id: crates/erp-hr/src/attendance/new_1
language: rust
---

# new

Creates a bounded 10,000 capacity ingestion gateway.

## Signature

```rust
pub fn new(capacity: usize) -> (Self, mpsc::Receiver<BiometricPunch>)
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Creates a bounded 10,000 capacity ingestion gateway.
[must_use]

## Source
Lines 51–54 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
