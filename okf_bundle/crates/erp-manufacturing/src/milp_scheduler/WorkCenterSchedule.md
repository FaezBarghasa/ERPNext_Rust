---
okf_version: "0.2"
type: Class
title: WorkCenterSchedule
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-manufacturing/src/milp_scheduler.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/milp_scheduler/WorkCenterSchedule
language: rust
---

# WorkCenterSchedule

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct WorkCenterSchedule
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `center_id`
- `capacity_hours`
- `scheduled_sequence`
- `total_setup_time_hours`
- `total_makespan_hours`

## Source
Lines 16–22 in `crates/erp-manufacturing/src/milp_scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [milp_scheduler](/crates/erp-manufacturing/src/milp_scheduler.md) |
