---
okf_version: "0.2"
type: Function
title: name_similarity
description: Computes Jaro-Winkler-style character similarity between two names (0.0 to 1.0).
resource: crates/erp-trade/src/sanctions.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:26:15Z"
concept_id: crates/erp-trade/src/sanctions/name_similarity_1
language: rust
---

# name_similarity

Computes Jaro-Winkler-style character similarity between two names (0.0 to 1.0).

## Signature

```rust
pub fn name_similarity(a: &str, b: &str) -> f64
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes Jaro-Winkler-style character similarity between two names (0.0 to 1.0).
[must_use]

## Source
Lines 18–42 in `crates/erp-trade/src/sanctions.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sanctions](/crates/erp-trade/src/sanctions.md) |
