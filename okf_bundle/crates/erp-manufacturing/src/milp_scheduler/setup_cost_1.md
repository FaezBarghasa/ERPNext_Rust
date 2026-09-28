---
okf_version: "0.2"
type: Function
title: setup_cost
description: "Computes sequence-dependent changeover cost S(j, k) between consecutive products."
resource: crates/erp-manufacturing/src/milp_scheduler.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/milp_scheduler/setup_cost_1
language: rust
---

# setup_cost

Computes sequence-dependent changeover cost S(j, k) between consecutive products.

## Signature

```rust
pub fn setup_cost(prod_a: &str, prod_b: &str) -> f64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes sequence-dependent changeover cost S(j, k) between consecutive products.
[must_use]

## Source
Lines 29–35 in `crates/erp-manufacturing/src/milp_scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [milp_scheduler](/crates/erp-manufacturing/src/milp_scheduler.md) |
