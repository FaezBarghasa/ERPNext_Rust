---
okf_version: "0.2"
type: Class
title: ProductionJob
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
concept_id: crates/erp-manufacturing/src/milp_scheduler/ProductionJob
language: rust
---

# ProductionJob

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ProductionJob
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `job_id`
- `product_code`
- `processing_time_hours`
- `due_date_hours`
- `tardiness_penalty_weight`
- `required_certification`

## Source
Lines 6–13 in `crates/erp-manufacturing/src/milp_scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [milp_scheduler](/crates/erp-manufacturing/src/milp_scheduler.md) |
