---
okf_version: "0.2"
type: Function
title: test_fifo_multi_batch_consumption
description: "[test]"
resource: crates/erp-inventory/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:57Z"
concept_id: crates/erp-inventory/src/lib/test_fifo_multi_batch_consumption
language: rust
---

# test_fifo_multi_batch_consumption

[test]

## Signature

```rust
fn test_fifo_multi_batch_consumption()
```

## Decorators

- `test`

## Docstring

[test]

## Source
Lines 19–43 in `crates/erp-inventory/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/erp-inventory/src/lib.md) |
| calls | [consume_fifo](/crates/erp-inventory/src/fifo/consume_fifo.md) |
