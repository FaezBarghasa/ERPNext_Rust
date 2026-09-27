---
okf_version: "0.2"
type: Class
title: JournalEntryLine
description: An individual debit/credit line item in a Journal Entry.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/JournalEntryLine
language: rust
---

# JournalEntryLine

An individual debit/credit line item in a Journal Entry.

## Signature

```rust
pub struct JournalEntryLine
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

An individual debit/credit line item in a Journal Entry.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `account`
- `debit`
- `credit`
- `debit_in_account_currency`
- `credit_in_account_currency`
- `exchange_rate`
- `party_type`
- `party`

## Source
Lines 37–54 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
