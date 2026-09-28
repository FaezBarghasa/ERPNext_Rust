---
okf_version: "0.2"
type: Function
title: can_accommodate
description: "[must_use]"
resource: crates/erp-wms/src/grid.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/grid/can_accommodate_1
language: rust
---

# can_accommodate

[must_use]

## Signature

```rust
pub fn can_accommodate(
        &self,
        weight: f64,
        volume: f64,
        hazmat: &HazmatClass,
        temp: &ThermalZone,
    ) -> bool
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 54–74 in `crates/erp-wms/src/grid.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [grid](/crates/erp-wms/src/grid.md) |
