---
okf_version: "0.2"
type: Function
title: generate_schedule
description: Generates complete amortization schedule with zero-loss final fractional cent reconciliation.
resource: crates/erp-lending/src/amortization.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-lending"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:48:02Z"
concept_id: crates/erp-lending/src/amortization/generate_schedule
language: rust
---

# generate_schedule

Generates complete amortization schedule with zero-loss final fractional cent reconciliation.

## Signature

```rust
impl AmortizationEngine { pub fn generate_schedule(
        principal: Decimal,
        annual_interest_rate: Decimal,
        total_periods: usize,
        start_date: NaiveDate,
    ) -> Result<Vec<AmortizationPeriod>, LendingError> }
```

## Visibility

- `pub`

## Docstring

Generates complete amortization schedule with zero-loss final fractional cent reconciliation.

## Source
Lines 63–114 in `crates/erp-lending/src/amortization.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amortization](/crates/erp-lending/src/amortization.md) |
