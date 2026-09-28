---
okf_version: "0.2"
type: Function
title: poisson_probability
description: "Computes Poisson probability P(k failures) = ((lambda*t)^k * e^(-lambda*t)) / k!"
resource: crates/erp-asset/src/mro.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:44:29Z"
concept_id: crates/erp-asset/src/mro/poisson_probability_1
language: rust
---

# poisson_probability

Computes Poisson probability P(k failures) = ((lambda*t)^k * e^(-lambda*t)) / k!

## Signature

```rust
pub fn poisson_probability(lambda_t: f64, k: u64) -> f64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes Poisson probability P(k failures) = ((lambda*t)^k * e^(-lambda*t)) / k!
[must_use]

## Source
Lines 28–35 in `crates/erp-asset/src/mro.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mro](/crates/erp-asset/src/mro.md) |
