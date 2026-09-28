---
okf_version: "0.2"
type: Function
title: test_warehouse_gl_sync_and_journal_balance
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
concept_id: crates/erp-inventory/src/lib/test_warehouse_gl_sync_and_journal_balance
language: rust
---

# test_warehouse_gl_sync_and_journal_balance

[test]

## Signature

```rust
fn test_warehouse_gl_sync_and_journal_balance()
```

## Decorators

- `test`

## Docstring

[test]

## Source
Lines 46–66 in `crates/erp-inventory/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/erp-inventory/src/lib.md) |
| calls | [create_purchase_receipt_gl_entries](/crates/erp-inventory/src/warehouse/create_purchase_receipt_gl_entries.md) |
| calls | [gl_entries_to_journal_entry](/crates/erp-inventory/src/warehouse/gl_entries_to_journal_entry.md) |
| calls | [create_delivery_note_gl_entries](/crates/erp-inventory/src/warehouse/create_delivery_note_gl_entries.md) |
