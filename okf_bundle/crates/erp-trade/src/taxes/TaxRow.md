---
okf_version: "0.2"
type: Class
title: TaxRow
description: Tax schedule row template.
resource: crates/erp-trade/src/taxes.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:22:38Z"
concept_id: crates/erp-trade/src/taxes/TaxRow
language: rust
---

# TaxRow

Tax schedule row template.

## Signature

```rust
pub struct TaxRow
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Tax schedule row template.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `account_head`
- `rate`
- `charge_type`

## Source
Lines 18–25 in `crates/erp-trade/src/taxes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [taxes](/crates/erp-trade/src/taxes.md) |
