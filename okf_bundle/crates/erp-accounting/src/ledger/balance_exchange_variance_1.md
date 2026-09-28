---
okf_version: "0.2"
type: Function
title: balance_exchange_variance
description: Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/ledger/balance_exchange_variance_1
language: rust
---

# balance_exchange_variance

Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.

## Signature

```rust
pub fn balance_exchange_variance(
        &mut self,
        gain_loss_account: &str,
    ) -> Result<(), AccountingError>
```

## Visibility

- `pub`

## Docstring

Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.

## Source
Lines 96–134 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
