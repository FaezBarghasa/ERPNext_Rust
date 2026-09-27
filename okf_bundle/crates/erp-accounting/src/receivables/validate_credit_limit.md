---
okf_version: "0.2"
type: Function
title: validate_credit_limit
description: "Validates customer credit limit constraint:"
resource: crates/erp-accounting/src/receivables.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:10:14Z"
concept_id: crates/erp-accounting/src/receivables/validate_credit_limit
language: rust
---

# validate_credit_limit

Validates customer credit limit constraint:

## Signature

```rust
impl ArApEngine { pub fn validate_credit_limit(
        current_outstanding: Decimal,
        unbilled_orders: Decimal,
        new_invoice_amount: Decimal,
        credit_limit: Decimal,
    ) -> Result<(), AccountingError> }
```

## Visibility

- `pub`

## Docstring

Validates customer credit limit constraint:
$\text{Current Outstanding} + \text{Unbilled Orders} + \text{New Invoice Amount} \le \text{Credit Limit}$

## Source
Lines 78–93 in `crates/erp-accounting/src/receivables.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [receivables](/crates/erp-accounting/src/receivables.md) |
