---
okf_version: "0.2"
type: Function
title: straight_line_dec
description: Straight-line depreciation in Decimal.
resource: crates/erp-accounting/src/decimal_ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/decimal_ledger/straight_line_dec
language: rust
---

# straight_line_dec

Straight-line depreciation in Decimal.

## Signature

```rust
pub fn straight_line_dec(cost: Decimal, salvage: Decimal, n: usize) -> Vec<Decimal>
```

## Visibility

- `pub`

## Docstring

Straight-line depreciation in Decimal.

## Source
Lines 16–27 in `crates/erp-accounting/src/decimal_ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [decimal_ledger](/crates/erp-accounting/src/decimal_ledger.md) |
