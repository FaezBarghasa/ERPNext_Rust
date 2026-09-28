---
okf_version: "0.2"
type: Function
title: calculate_pmt
description: Computes the fixed monthly installment PMT using exact fixed-point decimal math.
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:46:37Z"
concept_id: crates/erp-lending/src/amortization/calculate_pmt_1
language: rust
---

# calculate_pmt

Computes the fixed monthly installment PMT using exact fixed-point decimal math.

## Signature

```rust
pub fn calculate_pmt(
        principal: Decimal,
        annual_interest_rate: Decimal,
        total_periods: usize,
    ) -> Decimal
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes the fixed monthly installment PMT using exact fixed-point decimal math.
$$PMT = P \times \frac{r(1+r)^n}{(1+r)^n - 1}$$
[must_use]

## Source
Lines 43–64 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
