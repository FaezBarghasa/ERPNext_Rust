---
okf_version: "0.2"
type: Function
title: calculate_net_profit
description: "Computes Net Profit ($\\sum \\text{Income} - \\sum \\text{Expense}$)."
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/calculate_net_profit_1
language: rust
---

# calculate_net_profit

Computes Net Profit ($\sum \text{Income} - \sum \text{Expense}$).

## Signature

```rust
pub fn calculate_net_profit(
        entries: &[GlEntry],
        accounts: &HashMap<String, Account>,
        as_of: NaiveDate,
    ) -> Decimal
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes Net Profit ($\sum \text{Income} - \sum \text{Expense}$).
[must_use]

## Source
Lines 272–293 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
