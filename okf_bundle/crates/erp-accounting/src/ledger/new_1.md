---
okf_version: "0.2"
type: Function
title: new
description: Creates a new posting engine instance.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/new_1
language: rust
---

# new

Creates a new posting engine instance.

## Signature

```rust
pub fn new(accounts: Vec<Account>, closing_logs: Vec<PeriodClosingLog>) -> Self
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Creates a new posting engine instance.
[must_use]

## Source
Lines 176–185 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
