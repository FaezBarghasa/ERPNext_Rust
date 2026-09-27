---
okf_version: "0.2"
type: Function
title: add_fifo_layer
description: Adds incoming receipt batch layer to FIFO queue.
resource: crates/erp-inventory/src/fifo.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:16:39Z"
concept_id: crates/erp-inventory/src/fifo/add_fifo_layer
language: rust
---

# add_fifo_layer

Adds incoming receipt batch layer to FIFO queue.

## Signature

```rust
pub fn add_fifo_layer(queue: &mut Vec<FifoBatchItem>, qty: Decimal, rate: Decimal)
```

## Visibility

- `pub`

## Docstring

Adds incoming receipt batch layer to FIFO queue.

## Source
Lines 110–114 in `crates/erp-inventory/src/fifo.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [fifo](/crates/erp-inventory/src/fifo.md) |
