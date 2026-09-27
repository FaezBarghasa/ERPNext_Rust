---
okf_version: "0.2"
type: Function
title: calculate_net_requirement
description: "Calculates net production and procurement requirements:"
resource: crates/erp-manufacturing/src/mrp.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:43Z"
concept_id: crates/erp-manufacturing/src/mrp/calculate_net_requirement
language: rust
---

# calculate_net_requirement

Calculates net production and procurement requirements:

## Signature

```rust
impl MrpEngine { pub fn calculate_net_requirement(
        gross_demand: Decimal,
        current_stock: Decimal,
        open_purchase_orders: Decimal,
        safety_stock: Decimal,
    ) -> Decimal }
```

## Visibility

- `pub`

## Docstring

Calculates net production and procurement requirements:
$\text{Net Required} = \text{Gross Demand} - \text{Current Stock} - \text{Open PO} + \text{Safety Stock}$
[must_use]

## Source
Lines 33–41 in `crates/erp-manufacturing/src/mrp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mrp](/crates/erp-manufacturing/src/mrp.md) |
