---
okf_version: "0.2"
type: Function
title: straight_line_depreciation
description: Straight-line depreciation schedule generator using fixed-point decimal arithmetic.
resource: crates/erp-accounting/src/assets.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:11:11Z"
concept_id: crates/erp-accounting/src/assets/straight_line_depreciation
language: rust
---

# straight_line_depreciation

Straight-line depreciation schedule generator using fixed-point decimal arithmetic.

## Signature

```rust
pub fn straight_line_depreciation(
    cost: Decimal,
    salvage: Decimal,
    life_periods: usize,
) -> Vec<Decimal>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Straight-line depreciation schedule generator using fixed-point decimal arithmetic.
[must_use]

## Source
Lines 5–27 in `crates/erp-accounting/src/assets.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [assets](/crates/erp-accounting/src/assets.md) |
| called_by | [test_depreciation_zero_loss](/crates/erp-accounting/src/assets/test_depreciation_zero_loss.md) |
