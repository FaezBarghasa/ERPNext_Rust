---
okf_version: "0.2"
type: Function
title: recommended_safety_stock
description: "Calculates required safety stock quantity S such that cumulative sum P(k <= S) >= target_service_level."
resource: crates/erp-asset/src/mro.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:44:29Z"
concept_id: crates/erp-asset/src/mro/recommended_safety_stock_1
language: rust
---

# recommended_safety_stock

Calculates required safety stock quantity S such that cumulative sum P(k <= S) >= target_service_level.

## Signature

```rust
pub fn recommended_safety_stock(req: &MroPartRequirement) -> u64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Calculates required safety stock quantity S such that cumulative sum P(k <= S) >= target_service_level.
[must_use]

## Source
Lines 39–54 in `crates/erp-asset/src/mro.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mro](/crates/erp-asset/src/mro.md) |
