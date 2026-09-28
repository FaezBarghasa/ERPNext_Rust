---
okf_version: "0.2"
type: Function
title: are_compatible
description: Evaluates if two hazard classes can be co-stored within the same aisle or drainage basin.
resource: crates/erp-wms/src/grid.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/grid/are_compatible
language: rust
---

# are_compatible

Evaluates if two hazard classes can be co-stored within the same aisle or drainage basin.

## Signature

```rust
impl HazmatMatrix { pub fn are_compatible(a: &HazmatClass, b: &HazmatClass) -> bool }
```

## Visibility

- `pub`

## Docstring

Evaluates if two hazard classes can be co-stored within the same aisle or drainage basin.
[must_use]

## Source
Lines 82–105 in `crates/erp-wms/src/grid.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [grid](/crates/erp-wms/src/grid.md) |
