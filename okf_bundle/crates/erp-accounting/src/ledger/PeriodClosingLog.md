---
okf_version: "0.2"
type: Class
title: PeriodClosingLog
description: Period closing record sealing fiscal transactions.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/PeriodClosingLog
language: rust
---

# PeriodClosingLog

Period closing record sealing fiscal transactions.

## Signature

```rust
pub struct PeriodClosingLog
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Period closing record sealing fiscal transactions.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `company`
- `closing_date`

## Source
Lines 160–165 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
