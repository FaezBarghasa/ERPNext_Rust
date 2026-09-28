---
okf_version: "0.2"
type: Function
title: optimize_schedule
description: Solves sequence optimization using greedy heuristic with changeover minimization.
resource: crates/erp-manufacturing/src/milp_scheduler.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/milp_scheduler/optimize_schedule
language: rust
---

# optimize_schedule

Solves sequence optimization using greedy heuristic with changeover minimization.

## Signature

```rust
impl SequenceOptimizer { pub fn optimize_schedule(
        center_id: &str,
        capacity: f64,
        jobs: &[ProductionJob],
    ) -> WorkCenterSchedule }
```

## Visibility

- `pub`

## Docstring

Solves sequence optimization using greedy heuristic with changeover minimization.

## Source
Lines 38–95 in `crates/erp-manufacturing/src/milp_scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [milp_scheduler](/crates/erp-manufacturing/src/milp_scheduler.md) |
