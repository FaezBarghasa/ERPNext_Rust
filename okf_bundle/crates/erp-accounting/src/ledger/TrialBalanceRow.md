---
okf_version: "0.2"
type: Class
title: TrialBalanceRow
description: Trial Balance row summary.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/TrialBalanceRow
language: rust
---

# TrialBalanceRow

Trial Balance row summary.

## Signature

```rust
pub struct TrialBalanceRow
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Trial Balance row summary.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `account`
- `total_debit`
- `total_credit`
- `balance`

## Source
Lines 230–235 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
