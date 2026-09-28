---
okf_version: "0.2"
type: Function
title: post_journal_entry
description: Posts a journal entry to the immutable general ledger.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/ledger/post_journal_entry
language: rust
---

# post_journal_entry

Posts a journal entry to the immutable general ledger.

## Signature

```rust
impl LedgerPostingEngine { pub fn post_journal_entry(
        &self,
        entry: &JournalEntry,
        voucher_no: &str,
    ) -> Result<Vec<GlEntry>, AccountingError> }
```

## Visibility

- `pub`

## Docstring

Posts a journal entry to the immutable general ledger.

## Source
Lines 192–229 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
