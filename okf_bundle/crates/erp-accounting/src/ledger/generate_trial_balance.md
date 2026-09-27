---
okf_version: "0.2"
type: Function
title: generate_trial_balance
description: Generates real-time Trial Balance.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/generate_trial_balance
language: rust
---

# generate_trial_balance

Generates real-time Trial Balance.

## Signature

```rust
impl StatementGenerator { pub fn generate_trial_balance(entries: &[GlEntry], as_of: NaiveDate) -> Vec<TrialBalanceRow> }
```

## Visibility

- `pub`

## Docstring

Generates real-time Trial Balance.
[must_use]

## Source
Lines 243–268 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
