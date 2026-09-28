---
okf_version: "0.2"
type: Function
title: repayment_to_journal_entry
description: Converts repayment entries into a validated JournalEntry document.
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:46:37Z"
concept_id: crates/erp-lending/src/amortization/repayment_to_journal_entry_1
language: rust
---

# repayment_to_journal_entry

Converts repayment entries into a validated JournalEntry document.

## Signature

```rust
pub fn repayment_to_journal_entry(
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

Converts repayment entries into a validated JournalEntry document.
[must_use]

## Source
Lines 224–250 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
