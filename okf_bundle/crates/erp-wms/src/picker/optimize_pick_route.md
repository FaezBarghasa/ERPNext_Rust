---
okf_version: "0.2"
type: Function
title: optimize_pick_route
description: Computes TSP route using Nearest Neighbor with 2-Opt local search refinement.
resource: crates/erp-wms/src/picker.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/picker/optimize_pick_route
language: rust
---

# optimize_pick_route

Computes TSP route using Nearest Neighbor with 2-Opt local search refinement.

## Signature

```rust
impl PickPathOptimizer { pub fn optimize_pick_route(
        depot: &PickLocation,
        picks: &[PickLocation],
    ) -> (Vec<PickLocation>, f64) }
```

## Visibility

- `pub`

## Docstring

Computes TSP route using Nearest Neighbor with 2-Opt local search refinement.

## Source
Lines 29–90 in `crates/erp-wms/src/picker.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [picker](/crates/erp-wms/src/picker.md) |
