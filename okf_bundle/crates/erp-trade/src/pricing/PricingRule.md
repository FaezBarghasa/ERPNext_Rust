---
okf_version: "0.2"
type: Class
title: PricingRule
description: Dynamic Pricing Rule definition with priority-weighted discount matrix (Milestone 2.9).
resource: crates/erp-trade/src/pricing.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:48:48Z"
concept_id: crates/erp-trade/src/pricing/PricingRule
language: rust
---

# PricingRule

Dynamic Pricing Rule definition with priority-weighted discount matrix (Milestone 2.9).

## Signature

```rust
pub struct PricingRule
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Dynamic Pricing Rule definition with priority-weighted discount matrix (Milestone 2.9).
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `customer_group`
- `item_group`
- `min_qty`
- `max_qty`
- `priority`
- `discount_percentage`
- `discount_amount`

## Source
Lines 6–23 in `crates/erp-trade/src/pricing.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pricing](/crates/erp-trade/src/pricing.md) |
