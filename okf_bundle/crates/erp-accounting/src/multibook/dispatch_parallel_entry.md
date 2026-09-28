---
okf_version: "0.2"
type: Function
title: dispatch_parallel_entry
description: Concurrently generates balanced journal entries across all target accounting books.
resource: crates/erp-accounting/src/multibook.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/multibook/dispatch_parallel_entry
language: rust
---

# dispatch_parallel_entry

Concurrently generates balanced journal entries across all target accounting books.

## Signature

```rust
impl MultiBookPostingEngine { pub fn dispatch_parallel_entry(
        tx_id: &str,
        date: chrono::NaiveDate,
        company: &str,
        base_lines: &[MultiBookJournalLine],
        memo: &str,
    ) -> Result<Vec<MultiBookTransaction>, String> }
```

## Visibility

- `pub`

## Docstring

Concurrently generates balanced journal entries across all target accounting books.

## Source
Lines 44–76 in `crates/erp-accounting/src/multibook.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [multibook](/crates/erp-accounting/src/multibook.md) |
