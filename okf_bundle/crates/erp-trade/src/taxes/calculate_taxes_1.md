---
okf_version: "0.2"
type: Function
title: calculate_taxes
description: Computes compounding multi-tier taxes across tax schedule rows.
resource: crates/erp-trade/src/taxes.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-trade/src/taxes/calculate_taxes_1
language: rust
---

# calculate_taxes

Computes compounding multi-tier taxes across tax schedule rows.

## Signature

```rust
pub fn calculate_taxes(base_net_amount: Decimal, tax_rows: &[TaxRow]) -> TaxScheduleResult
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes compounding multi-tier taxes across tax schedule rows.
[must_use]

## Source
Lines 59–92 in `crates/erp-trade/src/taxes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [taxes](/crates/erp-trade/src/taxes.md) |
