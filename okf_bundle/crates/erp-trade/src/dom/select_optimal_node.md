---
okf_version: "0.2"
type: Function
title: select_optimal_node
description: Selects optimal fulfillment node minimizing total landed cost (Freight + Handling + Tax).
resource: crates/erp-trade/src/dom.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-trade/src/dom/select_optimal_node
language: rust
---

# select_optimal_node

Selects optimal fulfillment node minimizing total landed cost (Freight + Handling + Tax).

## Signature

```rust
impl DomRouter { pub fn select_optimal_node(
        required_qty: Decimal,
        nodes: &'a [FulfillmentNode],
        shipping_estimates: &[ShippingRateEstimate],
        item_unit_value: Decimal,
    ) -> Option<(&'a FulfillmentNode, Decimal)> }
```

## Type Parameters

- `'a`

## Visibility

- `pub`

## Docstring

Selects optimal fulfillment node minimizing total landed cost (Freight + Handling + Tax).

## Source
Lines 27–56 in `crates/erp-trade/src/dom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [dom](/crates/erp-trade/src/dom.md) |
