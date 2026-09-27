---
okf_version: "0.2"
type: Class
title: AmortizationPeriod
description: An individual installment period in an amortization schedule.
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:48:02Z"
concept_id: crates/erp-lending/src/amortization/AmortizationPeriod
language: rust
---

# AmortizationPeriod

An individual installment period in an amortization schedule.

## Signature

```rust
pub struct AmortizationPeriod
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

An individual installment period in an amortization schedule.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `period_no`
- `payment_date`
- `installment_amount`
- `principal_portion`
- `interest_portion`
- `remaining_balance`

## Source
Lines 21–34 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
