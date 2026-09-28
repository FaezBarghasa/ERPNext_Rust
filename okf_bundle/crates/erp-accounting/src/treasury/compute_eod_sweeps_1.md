---
okf_version: "0.2"
type: Function
title: compute_eod_sweeps
description: Evaluates end-of-day subsidiary accounts and sweeps excess cash into parent concentration account.
resource: crates/erp-accounting/src/treasury.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/treasury/compute_eod_sweeps_1
language: rust
---

# compute_eod_sweeps

Evaluates end-of-day subsidiary accounts and sweeps excess cash into parent concentration account.

## Signature

```rust
pub fn compute_eod_sweeps(
        subsidiary_accounts: &mut [BankAccount],
        parent_account_id: &str,
        intercompany_interest_rate: Decimal,
    ) -> Vec<ZbaSweepTransaction>
```

## Visibility

- `pub`

## Docstring

Evaluates end-of-day subsidiary accounts and sweeps excess cash into parent concentration account.

## Source
Lines 28–52 in `crates/erp-accounting/src/treasury.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [treasury](/crates/erp-accounting/src/treasury.md) |
