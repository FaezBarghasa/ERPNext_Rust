---
okf_version: "0.2"
type: Function
title: schedule_operation
description: "Schedules an operation interval, automatically sliding forward if a collision occurs."
resource: crates/erp-manufacturing/src/mrp.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:43Z"
concept_id: crates/erp-manufacturing/src/mrp/schedule_operation_1
language: rust
---

# schedule_operation

Schedules an operation interval, automatically sliding forward if a collision occurs.

## Signature

```rust
pub fn schedule_operation(
        &mut self,
        workstation: &str,
        job_id: String,
        mut desired_start: u64,
        duration: u64,
    ) -> ScheduledSlot
```

## Visibility

- `pub`

## Docstring

Schedules an operation interval, automatically sliding forward if a collision occurs.
Returns the assigned `ScheduledSlot`.

## Source
Lines 99–129 in `crates/erp-manufacturing/src/mrp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mrp](/crates/erp-manufacturing/src/mrp.md) |
