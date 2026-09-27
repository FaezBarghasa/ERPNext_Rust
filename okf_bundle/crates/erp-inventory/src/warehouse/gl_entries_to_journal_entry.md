---
okf_version: "0.2"
type: Function
title: gl_entries_to_journal_entry
description: Helper converting GL entries into a validated JournalEntry document.
resource: crates/erp-inventory/src/warehouse.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:08Z"
concept_id: crates/erp-inventory/src/warehouse/gl_entries_to_journal_entry
language: rust
---

# gl_entries_to_journal_entry

Helper converting GL entries into a validated JournalEntry document.

## Signature

```rust
pub fn gl_entries_to_journal_entry(
    entries: &[GlEntry],
    posting_date: NaiveDate,
    company: &str,
    remarks: &str,
) -> JournalEntry
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Helper converting GL entries into a validated JournalEntry document.
[must_use]

## Source
Lines 99–125 in `crates/erp-inventory/src/warehouse.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [warehouse](/crates/erp-inventory/src/warehouse.md) |
| called_by | [test_warehouse_gl_sync_and_journal_balance](/crates/erp-inventory/src/lib/test_warehouse_gl_sync_and_journal_balance.md) |
