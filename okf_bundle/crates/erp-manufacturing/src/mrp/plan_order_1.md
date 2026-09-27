---
okf_version: "0.2"
type: Function
title: plan_order
description: Generates a planned replenishment order.
resource: crates/erp-manufacturing/src/mrp.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:43Z"
concept_id: crates/erp-manufacturing/src/mrp/plan_order_1
language: rust
---

# plan_order

Generates a planned replenishment order.

## Signature

```rust
pub fn plan_order(
        item_code: &str,
        gross_demand: Decimal,
        current_stock: Decimal,
        open_po: Decimal,
        safety_stock: Decimal,
        is_manufactured: bool,
        lead_time_days: u32,
    ) -> Option<PlannedOrder>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates a planned replenishment order.
[must_use]

## Source
Lines 45–69 in `crates/erp-manufacturing/src/mrp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mrp](/crates/erp-manufacturing/src/mrp.md) |
