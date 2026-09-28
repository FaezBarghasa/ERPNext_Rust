---
okf_version: "0.2"
type: Class
title: ScheduleTask
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-ppm/src/scheduler.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/scheduler/ScheduleTask
language: rust
---

# ScheduleTask

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ScheduleTask
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `name`
- `duration`
- `dependencies`
- `early_start`
- `early_finish`
- `late_start`
- `late_finish`
- `total_float`
- `free_float`
- `is_critical`

## Source
Lines 21–33 in `crates/erp-ppm/src/scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scheduler](/crates/erp-ppm/src/scheduler.md) |
