---
okf_version: "0.2"
type: Class
title: AccountingError
description: Accounting domain errors.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/AccountingError
language: rust
---

# AccountingError

Accounting domain errors.

## Signature

```rust
pub enum AccountingError
```

## Decorators

- `derive(Debug, Error, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Accounting domain errors.
[derive(Debug, Error, PartialEq, Eq)]

## Methods

- `total_debit`
- `total_credit`
- `current`
- `new_amount`
- `limit`

## Source
Lines 10–33 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
