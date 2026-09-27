---
okf_version: "0.2"
type: Function
title: consume_fifo
description: "Consumes quantities from a FIFO batch queue sequentially, calculating exact COGS."
resource: crates/erp-inventory/src/fifo.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:16:39Z"
concept_id: crates/erp-inventory/src/fifo/consume_fifo
language: rust
---

# consume_fifo

Consumes quantities from a FIFO batch queue sequentially, calculating exact COGS.

## Signature

```rust
pub fn consume_fifo(
    queue: &mut Vec<FifoBatchItem>,
    mut qty_to_remove: Decimal,
) -> Result<Decimal, InventoryError>
```

## Visibility

- `pub`

## Docstring

Consumes quantities from a FIFO batch queue sequentially, calculating exact COGS.

## Source
Lines 71–107 in `crates/erp-inventory/src/fifo.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [fifo](/crates/erp-inventory/src/fifo.md) |
| called_by | [test_fifo_multi_batch_consumption](/crates/erp-inventory/src/lib/test_fifo_multi_batch_consumption.md) |
