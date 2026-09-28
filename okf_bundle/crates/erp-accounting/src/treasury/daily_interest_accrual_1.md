---
okf_version: "0.2"
type: Function
title: daily_interest_accrual
description: "Computes daily intercompany interest accrual: (Loan Principal * Annual Rate) / 365."
resource: crates/erp-accounting/src/treasury.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/treasury/daily_interest_accrual_1
language: rust
---

# daily_interest_accrual

Computes daily intercompany interest accrual: (Loan Principal * Annual Rate) / 365.

## Signature

```rust
pub fn daily_interest_accrual(principal: Decimal, annual_rate: Decimal) -> Decimal
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes daily intercompany interest accrual: (Loan Principal * Annual Rate) / 365.
[must_use]

## Source
Lines 56–58 in `crates/erp-accounting/src/treasury.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [treasury](/crates/erp-accounting/src/treasury.md) |
