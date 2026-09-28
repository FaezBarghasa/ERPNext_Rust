---
okf_version: "0.2"
type: Function
title: new
description: "[must_use]"
resource: crates/erp-manufacturing/src/eco.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:16:42Z"
concept_id: crates/erp-manufacturing/src/eco/new_1
language: rust
---

# new

[must_use]

## Signature

```rust
pub fn new(
        eco_number: String,
        item_code: String,
        current_rev: String,
        target_rev: String,
        reason: String,
        disposition: DispositionMode,
        effective_date: chrono::NaiveDate,
    ) -> Self
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 37–57 in `crates/erp-manufacturing/src/eco.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [eco](/crates/erp-manufacturing/src/eco.md) |
