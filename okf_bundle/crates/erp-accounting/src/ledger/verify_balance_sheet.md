---
okf_version: "0.2"
type: Function
title: verify_balance_sheet
description: "Verifies the Fundamental Accounting Equation: $\\text{Assets} = \\text{Liabilities} + \\text{Equity} + \\text{Net Profit}$."
resource: crates/erp-accounting/src/ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:45Z"
concept_id: crates/erp-accounting/src/ledger/verify_balance_sheet
language: rust
---

# verify_balance_sheet

Verifies the Fundamental Accounting Equation: $\text{Assets} = \text{Liabilities} + \text{Equity} + \text{Net Profit}$.

## Signature

```rust
impl StatementGenerator { pub fn verify_balance_sheet(
        entries: &[GlEntry],
        accounts: &HashMap<String, Account>,
        as_of: NaiveDate,
    ) -> bool }
```

## Visibility

- `pub`

## Docstring

Verifies the Fundamental Accounting Equation: $\text{Assets} = \text{Liabilities} + \text{Equity} + \text{Net Profit}$.
[must_use]

## Source
Lines 297–321 in `crates/erp-accounting/src/ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ledger](/crates/erp-accounting/src/ledger.md) |
