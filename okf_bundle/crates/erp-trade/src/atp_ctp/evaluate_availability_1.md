---
okf_version: "0.2"
type: Function
title: evaluate_availability
description: "[must_use]"
resource: crates/erp-trade/src/atp_ctp.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-trade/src/atp_ctp/evaluate_availability_1
language: rust
---

# evaluate_availability

[must_use]

## Signature

```rust
pub fn evaluate_availability(
        pos: &InventoryPosition,
        requested_qty: Decimal,
    ) -> PromiseAvailability
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 28–46 in `crates/erp-trade/src/atp_ctp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [atp_ctp](/crates/erp-trade/src/atp_ctp.md) |
