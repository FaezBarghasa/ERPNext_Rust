---
okf_version: "0.2"
type: Class
title: InventoryError
description: Inventory domain errors.
resource: crates/erp-inventory/src/fifo.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:16:39Z"
concept_id: crates/erp-inventory/src/fifo/InventoryError
language: rust
---

# InventoryError

Inventory domain errors.

## Signature

```rust
pub enum InventoryError
```

## Decorators

- `derive(Debug, Error, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Inventory domain errors.
[derive(Debug, Error, PartialEq, Eq)]

## Methods

- `requested`
- `available`
- `serial_no`
- `expected_warehouse`
- `actual_warehouse`
- `batch_id`
- `expiry_date`

## Source
Lines 8–34 in `crates/erp-inventory/src/fifo.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [fifo](/crates/erp-inventory/src/fifo.md) |
