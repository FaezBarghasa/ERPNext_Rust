---
okf_version: "0.2"
type: Function
title: reliability
description: Reliability function R(t) = exp(-((t - gamma) / eta)^beta)
resource: crates/erp-asset/src/reliability.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/reliability/reliability
language: rust
---

# reliability

Reliability function R(t) = exp(-((t - gamma) / eta)^beta)

## Signature

```rust
impl WeibullParameters { pub fn reliability(&self, t: f64) -> f64 }
```

## Visibility

- `pub`

## Docstring

Reliability function R(t) = exp(-((t - gamma) / eta)^beta)
[must_use]

## Source
Lines 20–26 in `crates/erp-asset/src/reliability.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [reliability](/crates/erp-asset/src/reliability.md) |
