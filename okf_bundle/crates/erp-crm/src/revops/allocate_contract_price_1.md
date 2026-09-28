---
okf_version: "0.2"
type: Function
title: allocate_contract_price
description: "5-Step Revenue Recognition: Allocates Transaction Price (TP) based on relative Standalone Selling Prices (SSP)."
resource: crates/erp-crm/src/revops.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/revops/allocate_contract_price_1
language: rust
---

# allocate_contract_price

5-Step Revenue Recognition: Allocates Transaction Price (TP) based on relative Standalone Selling Prices (SSP).

## Signature

```rust
pub fn allocate_contract_price(contract: &mut RevenueContract) -> Result<(), String>
```

## Visibility

- `pub`

## Docstring

5-Step Revenue Recognition: Allocates Transaction Price (TP) based on relative Standalone Selling Prices (SSP).

## Source
Lines 36–53 in `crates/erp-crm/src/revops.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [revops](/crates/erp-crm/src/revops.md) |
