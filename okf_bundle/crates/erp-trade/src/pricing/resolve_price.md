---
okf_version: "0.2"
type: Function
title: resolve_price
description: Resolves the applicable unit price by matching quantity and priority.
resource: crates/erp-trade/src/pricing.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:48:48Z"
concept_id: crates/erp-trade/src/pricing/resolve_price
language: rust
---

# resolve_price

Resolves the applicable unit price by matching quantity and priority.

## Signature

```rust
impl PricingEngine { pub fn resolve_price(
        base_rate: Decimal,
        qty: Decimal,
        customer_group: Option<&str>,
        item_group: Option<&str>,
        rules: &[PricingRule],
    ) -> Decimal }
```

## Visibility

- `pub`

## Docstring

Resolves the applicable unit price by matching quantity and priority.
[must_use]

## Source
Lines 31–73 in `crates/erp-trade/src/pricing.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pricing](/crates/erp-trade/src/pricing.md) |
