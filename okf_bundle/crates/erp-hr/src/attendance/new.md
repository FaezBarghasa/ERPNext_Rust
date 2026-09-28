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
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-hr/src/attendance/new
language: rust
---

# new

Creates a bounded 10,000 capacity ingestion gateway.

## Signature

```rust
impl BiometricIngestionGateway { pub fn new(capacity: usize) -> (Self, mpsc::Receiver<BiometricPunch>) }
```

## Visibility

- `pub`

## Docstring

Creates a bounded 10,000 capacity ingestion gateway.
[must_use]

## Source
Lines 53–56 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
