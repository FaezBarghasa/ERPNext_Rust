---
okf_version: "0.2"
type: Function
title: score_bin
description: Computes putaway fit score for a candidate bin (higher is better).
resource: crates/erp-wms/src/slotting.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/slotting/score_bin_1
language: rust
---

# score_bin

Computes putaway fit score for a candidate bin (higher is better).

## Signature

```rust
pub fn score_bin(item: &PutawayItem, bin: &WarehouseBin) -> Option<f64>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes putaway fit score for a candidate bin (higher is better).
[must_use]

## Source
Lines 28–53 in `crates/erp-wms/src/slotting.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [slotting](/crates/erp-wms/src/slotting.md) |
