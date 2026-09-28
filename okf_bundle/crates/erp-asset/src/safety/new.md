---
okf_version: "0.2"
type: Function
title: new
description: "[must_use]"
resource: crates/erp-asset/src/safety.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/safety/new
language: rust
---

# new

[must_use]

## Signature

```rust
impl PermitToWork { pub fn new(
        permit_id: String,
        asset_id: String,
        work_order_id: String,
        work_type: String,
    ) -> Self }
```

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 39–55 in `crates/erp-asset/src/safety.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [safety](/crates/erp-asset/src/safety.md) |
