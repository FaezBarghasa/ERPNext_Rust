---
okf_version: "0.2"
type: Class
title: GlEntry
description: Immutable General Ledger Entry row.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/GlEntry
language: rust
---

# GlEntry

Immutable General Ledger Entry row.

## Signature

```rust
pub struct GlEntry
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Immutable General Ledger Entry row.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `posting_date`
- `account`
- `debit`
- `credit`
- `voucher_type`
- `voucher_no`
- `party_type`
- `party`
- `company`

## Source
Lines 135–156 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
