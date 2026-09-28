---
okf_version: "0.2"
type: Class
title: IntercompanyTransaction
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-accounting/src/intercompany.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:22:07Z"
concept_id: crates/erp-accounting/src/intercompany/IntercompanyTransaction
language: rust
---

# IntercompanyTransaction

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct IntercompanyTransaction
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `transaction_id`
- `seller_entity_id`
- `buyer_entity_id`
- `amount`
- `cost_of_goods_sold`
- `currency`
- `is_inventory_movement`

## Source
Lines 7–15 in `crates/erp-accounting/src/intercompany.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [intercompany](/crates/erp-accounting/src/intercompany.md) |
