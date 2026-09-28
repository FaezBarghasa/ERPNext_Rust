---
okf_version: "0.2"
type: Class
title: PriceWaterfall
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-crm/src/cpq.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/cpq/PriceWaterfall
language: rust
---

# PriceWaterfall

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct PriceWaterfall
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `list_price`
- `volume_discount`
- `base_price`
- `contract_tier_discount`
- `net_price`
- `freight_and_tariffs`
- `landed_price`
- `early_pay_discount`
- `pocket_price`
- `target_margin_percent`
- `actual_margin_percent`
- `requires_executive_approval`

## Source
Lines 16–29 in `crates/erp-crm/src/cpq.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [cpq](/crates/erp-crm/src/cpq.md) |
