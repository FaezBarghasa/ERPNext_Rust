---
okf_version: "0.2"
type: Function
title: create_repayment_gl_entries
description: "Generates GL entries for incoming loan repayment installment:"
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:46:37Z"
concept_id: crates/erp-lending/src/amortization/create_repayment_gl_entries_1
language: rust
---

# create_repayment_gl_entries

Generates GL entries for incoming loan repayment installment:

## Signature

```rust
pub fn create_repayment_gl_entries(
        bank_account: &str,
        loan_principal_account: &str,
        interest_receivable_account: &str,
        principal_portion: Decimal,
        interest_portion: Decimal,
        posting_date: NaiveDate,
        voucher_no: &str,
        company: &str,
    ) -> Vec<GlEntry>
```

## Decorators

- `must_use`
- `allow(clippy::too_many_arguments)`

## Visibility

- `pub`

## Docstring

Generates GL entries for incoming loan repayment installment:
- Debit: Bank Account (Total Installment)
- Credit: Loan Principal Receivable (Principal Portion)
- Credit: Interest Receivable (Interest Portion)
[must_use]
[allow(clippy::too_many_arguments)]

## Source
Lines 171–220 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
