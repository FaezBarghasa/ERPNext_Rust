---
okf_version: "0.2"
type: Function
title: create_delivery_note_gl_entries
description: Generates balancing GL entries for a Delivery Note submission (Milestone 2.7).
resource: crates/erp-inventory/src/warehouse.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:08Z"
concept_id: crates/erp-inventory/src/warehouse/create_delivery_note_gl_entries
language: rust
---

# create_delivery_note_gl_entries

Generates balancing GL entries for a Delivery Note submission (Milestone 2.7).

## Signature

```rust
pub fn create_delivery_note_gl_entries(
    warehouse: &Warehouse,
    cogs_amount: Decimal,
    posting_date: NaiveDate,
    voucher_no: &str,
) -> Vec<GlEntry>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates balancing GL entries for a Delivery Note submission (Milestone 2.7).
- Debit: Cost of Goods Sold (Expense increases)
- Credit: Stock In Hand (Asset decreases)
[must_use]

## Source
Lines 63–95 in `crates/erp-inventory/src/warehouse.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [warehouse](/crates/erp-inventory/src/warehouse.md) |
| called_by | [test_warehouse_gl_sync_and_journal_balance](/crates/erp-inventory/src/lib/test_warehouse_gl_sync_and_journal_balance.md) |
