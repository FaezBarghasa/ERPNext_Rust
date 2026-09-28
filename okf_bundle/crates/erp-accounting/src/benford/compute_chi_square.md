---
okf_version: "0.2"
type: Function
title: compute_chi_square
description: Computes Chi-Square goodness-of-fit statistic against Benford distribution.
resource: crates/erp-accounting/src/benford.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:42:32Z"
concept_id: crates/erp-accounting/src/benford/compute_chi_square
language: rust
---

# compute_chi_square

Computes Chi-Square goodness-of-fit statistic against Benford distribution.

## Signature

```rust
impl BenfordGuard { pub fn compute_chi_square(amounts: &[Decimal]) -> f64 }
```

## Visibility

- `pub`

## Docstring

Computes Chi-Square goodness-of-fit statistic against Benford distribution.
Chi-Square > 15.51 (at df=8, p=0.05) indicates statistically significant anomaly.
[must_use]

## Source
Lines 20–51 in `crates/erp-accounting/src/benford.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [benford](/crates/erp-accounting/src/benford.md) |
