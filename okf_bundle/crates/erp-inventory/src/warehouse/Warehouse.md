---
okf_version: "0.2"
type: Class
title: Warehouse
description: Warehouse master record with linked General Ledger accounts.
resource: crates/erp-inventory/src/warehouse.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:08Z"
concept_id: crates/erp-inventory/src/warehouse/Warehouse
language: rust
---

# Warehouse

Warehouse master record with linked General Ledger accounts.

## Signature

```rust
pub struct Warehouse
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Warehouse master record with linked General Ledger accounts.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `stock_in_hand_account`
- `stock_received_but_not_billed_account`
- `default_cogs_account`
- `company`

## Source
Lines 8–19 in `crates/erp-inventory/src/warehouse.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [warehouse](/crates/erp-inventory/src/warehouse.md) |
