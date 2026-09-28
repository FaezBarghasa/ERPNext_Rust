---
okf_version: "0.2"
type: Function
title: expected_probability
description: "Expected probability of leading digit d under Benford's Law: P(d) = log10(1 + 1/d)"
resource: crates/erp-accounting/src/benford.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:42:32Z"
concept_id: crates/erp-accounting/src/benford/expected_probability_1
language: rust
---

# expected_probability

Expected probability of leading digit d under Benford's Law: P(d) = log10(1 + 1/d)

## Signature

```rust
pub fn expected_probability(digit: u8) -> f64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Expected probability of leading digit d under Benford's Law: P(d) = log10(1 + 1/d)
[must_use]

## Source
Lines 10–15 in `crates/erp-accounting/src/benford.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [benford](/crates/erp-accounting/src/benford.md) |
