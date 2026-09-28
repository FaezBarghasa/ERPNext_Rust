---
okf_version: "0.2"
type: Function
title: calculate_waterfall
description: Evaluates multi-tier price waterfall and margin protection thresholds.
resource: crates/erp-crm/src/cpq.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/cpq/calculate_waterfall_1
language: rust
---

# calculate_waterfall

Evaluates multi-tier price waterfall and margin protection thresholds.

## Signature

```rust
pub fn calculate_waterfall(
        &self,
        list_price: Decimal,
        volume_disc_pct: Decimal,
        tier_disc_pct: Decimal,
        tariffs: Decimal,
        early_pay_disc_pct: Decimal,
        total_cog: Decimal,
    ) -> PriceWaterfall
```

## Visibility

- `pub`

## Docstring

Evaluates multi-tier price waterfall and margin protection thresholds.

## Source
Lines 61–103 in `crates/erp-crm/src/cpq.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [cpq](/crates/erp-crm/src/cpq.md) |
