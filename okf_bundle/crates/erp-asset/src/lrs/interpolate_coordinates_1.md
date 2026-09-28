---
okf_version: "0.2"
type: Function
title: interpolate_coordinates
description: "[must_use]"
resource: crates/erp-asset/src/lrs.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/lrs/interpolate_coordinates_1
language: rust
---

# interpolate_coordinates

[must_use]

## Signature

```rust
pub fn interpolate_coordinates(
        start_marker: &LinearMarker,
        end_marker: &LinearMarker,
        target_chainage: f64,
    ) -> Option<(f64, f64)>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 41–60 in `crates/erp-asset/src/lrs.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lrs](/crates/erp-asset/src/lrs.md) |
