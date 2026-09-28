---
okf_version: "0.2"
type: Function
title: assign_order
description: Dispatches orders to best idle AMR robot based on battery level and proximity.
resource: crates/erp-wms/src/amr.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:43:54Z"
concept_id: crates/erp-wms/src/amr/assign_order_1
language: rust
---

# assign_order

Dispatches orders to best idle AMR robot based on battery level and proximity.

## Signature

```rust
pub fn assign_order(
        order: &Vda5050Order,
        robots: &'a mut [AmrTelemetry],
    ) -> Option<&'a mut AmrTelemetry>
```

## Type Parameters

- `'a`

## Visibility

- `pub`

## Docstring

Dispatches orders to best idle AMR robot based on battery level and proximity.

## Source
Lines 67–86 in `crates/erp-wms/src/amr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amr](/crates/erp-wms/src/amr.md) |
