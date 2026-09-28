---
okf_version: "0.2"
type: Function
title: calculate_charge
description: Rates a total volume of usage events under specified rating model.
resource: crates/erp-trade/src/metered.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-trade/src/metered/calculate_charge_1
language: rust
---

# calculate_charge

Rates a total volume of usage events under specified rating model.

## Signature

```rust
pub fn calculate_charge(total_units: Decimal, model: &RatingModel) -> Decimal
```

## Visibility

- `pub`

## Docstring

Rates a total volume of usage events under specified rating model.

## Source
Lines 29–72 in `crates/erp-trade/src/metered.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [metered](/crates/erp-trade/src/metered.md) |
