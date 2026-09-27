---
okf_version: "0.2"
type: Class
title: StockLedgerEntry
description: Immutable Stock Ledger Entry (SLE) recording physical inventory movement.
resource: crates/erp-inventory/src/fifo.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:16:39Z"
concept_id: crates/erp-inventory/src/fifo/StockLedgerEntry
language: rust
---

# StockLedgerEntry

Immutable Stock Ledger Entry (SLE) recording physical inventory movement.

## Signature

```rust
pub struct StockLedgerEntry
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Immutable Stock Ledger Entry (SLE) recording physical inventory movement.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `item_code`
- `warehouse`
- `actual_qty`
- `incoming_rate`
- `valuation_rate`
- `stock_value_difference`
- `posting_date`
- `voucher_type`
- `voucher_no`

## Source
Lines 47–68 in `crates/erp-inventory/src/fifo.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [fifo](/crates/erp-inventory/src/fifo.md) |
