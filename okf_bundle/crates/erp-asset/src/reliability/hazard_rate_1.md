---
okf_version: "0.2"
type: Function
title: hazard_rate
description: "Hazard rate / failure rate h(t) = (beta / eta) * ((t - gamma) / eta)^(beta - 1)"
resource: crates/erp-asset/src/reliability.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/reliability/hazard_rate_1
language: rust
---

# hazard_rate

Hazard rate / failure rate h(t) = (beta / eta) * ((t - gamma) / eta)^(beta - 1)

## Signature

```rust
pub fn hazard_rate(&self, t: f64) -> f64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Hazard rate / failure rate h(t) = (beta / eta) * ((t - gamma) / eta)^(beta - 1)
[must_use]

## Source
Lines 30–36 in `crates/erp-asset/src/reliability.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [reliability](/crates/erp-asset/src/reliability.md) |
