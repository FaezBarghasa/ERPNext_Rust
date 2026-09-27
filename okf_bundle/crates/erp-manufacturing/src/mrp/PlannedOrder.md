---
okf_version: "0.2"
type: Class
title: PlannedOrder
description: Generated MRP replenishment requirement plan.
resource: crates/erp-manufacturing/src/mrp.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:43Z"
concept_id: crates/erp-manufacturing/src/mrp/PlannedOrder
language: rust
---

# PlannedOrder

Generated MRP replenishment requirement plan.

## Signature

```rust
pub struct PlannedOrder
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Generated MRP replenishment requirement plan.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `item_code`
- `qty`
- `order_type`
- `lead_time_days`

## Source
Lines 15–24 in `crates/erp-manufacturing/src/mrp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mrp](/crates/erp-manufacturing/src/mrp.md) |
