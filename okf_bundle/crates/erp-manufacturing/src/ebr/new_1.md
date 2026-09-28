---
okf_version: "0.2"
type: Function
title: new
description: "[must_use]"
resource: crates/erp-manufacturing/src/ebr.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:49:30Z"
concept_id: crates/erp-manufacturing/src/ebr/new_1
language: rust
---

# new

[must_use]

## Signature

```rust
pub fn new(
        batch_no: String,
        product: String,
        mfg_date: chrono::NaiveDate,
        exp_date: chrono::NaiveDate,
    ) -> Self
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 37–51 in `crates/erp-manufacturing/src/ebr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ebr](/crates/erp-manufacturing/src/ebr.md) |
