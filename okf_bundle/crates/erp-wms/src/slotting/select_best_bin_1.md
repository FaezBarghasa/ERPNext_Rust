---
okf_version: "0.2"
type: Function
title: select_best_bin
description: Selects optimal candidate bin from a list.
resource: crates/erp-wms/src/slotting.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-wms/src/slotting/select_best_bin_1
language: rust
---

# select_best_bin

Selects optimal candidate bin from a list.

## Signature

```rust
pub fn select_best_bin(
        item: &PutawayItem,
        bins: &'a [WarehouseBin],
    ) -> Option<(&'a WarehouseBin, f64)>
```

## Type Parameters

- `'a`

## Visibility

- `pub`

## Docstring

Selects optimal candidate bin from a list.

## Source
Lines 56–69 in `crates/erp-wms/src/slotting.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [slotting](/crates/erp-wms/src/slotting.md) |
