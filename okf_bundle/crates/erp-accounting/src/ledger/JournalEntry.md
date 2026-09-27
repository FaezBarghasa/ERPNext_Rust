---
okf_version: "0.2"
type: Class
title: JournalEntry
description: A complete journal entry transaction document.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/JournalEntry
language: rust
---

# JournalEntry

A complete journal entry transaction document.

## Signature

```rust
pub struct JournalEntry
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

A complete journal entry transaction document.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `posting_date`
- `company`
- `lines`
- `remarks`

## Source
Lines 58–67 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
