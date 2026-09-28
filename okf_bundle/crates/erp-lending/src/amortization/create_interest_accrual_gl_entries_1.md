---
okf_version: "0.2"
type: Function
title: create_interest_accrual_gl_entries
description: "Generates GL entries for monthly interest accrual:"
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:46:37Z"
concept_id: crates/erp-lending/src/amortization/create_interest_accrual_gl_entries_1
language: rust
---

# create_interest_accrual_gl_entries

Generates GL entries for monthly interest accrual:

## Signature

```rust
pub fn create_interest_accrual_gl_entries(
        interest_receivable_account: &str,
        interest_income_account: &str,
        interest_amount: Decimal,
        posting_date: NaiveDate,
        voucher_no: &str,
        company: &str,
    ) -> Vec<GlEntry>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates GL entries for monthly interest accrual:
- Debit: Interest Receivable (Asset increases)
- Credit: Interest Income (Revenue increases)
[must_use]

## Source
Lines 129–163 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
