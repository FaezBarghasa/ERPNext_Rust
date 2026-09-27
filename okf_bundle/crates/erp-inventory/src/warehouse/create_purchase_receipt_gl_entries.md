---
okf_version: "0.2"
type: Function
title: create_purchase_receipt_gl_entries
description: Generates balancing GL entries for a Purchase Receipt submission (Milestone 2.7).
resource: crates/erp-inventory/src/warehouse.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:08Z"
concept_id: crates/erp-inventory/src/warehouse/create_purchase_receipt_gl_entries
language: rust
---

# create_purchase_receipt_gl_entries

Generates balancing GL entries for a Purchase Receipt submission (Milestone 2.7).

## Signature

```rust
pub fn create_purchase_receipt_gl_entries(
    warehouse: &Warehouse,
    total_valuation_amount: Decimal,
    posting_date: NaiveDate,
    voucher_no: &str,
) -> Vec<GlEntry>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates balancing GL entries for a Purchase Receipt submission (Milestone 2.7).
- Debit: Stock In Hand (Asset increases)
- Credit: Stock Received But Not Billed (Liability increases)
[must_use]

## Source
Lines 25–57 in `crates/erp-inventory/src/warehouse.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [warehouse](/crates/erp-inventory/src/warehouse.md) |
| called_by | [test_warehouse_gl_sync_and_journal_balance](/crates/erp-inventory/src/lib/test_warehouse_gl_sync_and_journal_balance.md) |
