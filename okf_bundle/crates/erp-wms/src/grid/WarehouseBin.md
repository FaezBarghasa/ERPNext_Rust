---
okf_version: "0.2"
type: Class
title: WarehouseBin
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-wms/src/grid.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/grid/WarehouseBin
language: rust
---

# WarehouseBin

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct WarehouseBin
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `site`
- `warehouse`
- `zone`
- `aisle`
- `bay`
- `level`
- `bin`
- `thermal_zone`
- `max_weight_kg`
- `max_volume_m3`
- `current_weight_kg`
- `current_volume_m3`
- `allowed_hazmat`

## Source
Lines 27–42 in `crates/erp-wms/src/grid.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [grid](/crates/erp-wms/src/grid.md) |
