---
okf_version: "0.2"
type: Function
title: distribute_landed_cost
description: Distributes landed cost charges across item valuation amounts proportionally.
resource: crates/erp-trade/src/landed_cost.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:24:01Z"
concept_id: crates/erp-trade/src/landed_cost/distribute_landed_cost
language: rust
---

# distribute_landed_cost

Distributes landed cost charges across item valuation amounts proportionally.

## Signature

```rust
pub fn distribute_landed_cost(item_values: &[Decimal], total_charges: Decimal) -> Vec<Decimal>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Distributes landed cost charges across item valuation amounts proportionally.
[must_use]

## Source
Lines 5–24 in `crates/erp-trade/src/landed_cost.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [landed_cost](/crates/erp-trade/src/landed_cost.md) |
| called_by | [test_landed_cost_distribution](/crates/erp-trade/src/landed_cost/test_landed_cost_distribution.md) |
