---
okf_version: "0.2"
type: Function
title: calculate
resource: crates/erp-ppm/src/construction.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/construction/calculate
language: rust
---

# calculate

## Signature

```rust
impl AiaG702Certificate { pub fn calculate(
        original_contract: Decimal,
        change_orders: &[ChangeOrder],
        sov_items: &[ScheduleOfValuesItem],
        previous_certificates_total: Decimal,
    ) -> Self }
```

## Visibility

- `pub`

## Source
Lines 94–130 in `crates/erp-ppm/src/construction.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [construction](/crates/erp-ppm/src/construction.md) |
