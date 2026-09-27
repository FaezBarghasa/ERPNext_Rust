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
timestamp: "2026-09-27T19:22:38Z"
concept_id: crates/erp-trade/src/taxes/calculate_taxes
language: rust
---

# calculate_taxes

Computes compounding multi-tier taxes across tax schedule rows.

## Signature

```rust
impl TaxEngine { pub fn calculate_taxes(base_net_amount: Decimal, tax_rows: &[TaxRow]) -> TaxScheduleResult }
```

## Visibility

- `pub`

## Docstring

Computes compounding multi-tier taxes across tax schedule rows.
[must_use]

## Source
Lines 59–94 in `crates/erp-trade/src/taxes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [taxes](/crates/erp-trade/src/taxes.md) |
