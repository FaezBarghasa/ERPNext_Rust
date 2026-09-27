---
okf_version: "0.2"
type: Class
title: TaxScheduleResult
description: Result of complete multi-tier tax calculation.
resource: crates/erp-trade/src/taxes.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:22:38Z"
concept_id: crates/erp-trade/src/taxes/TaxScheduleResult
language: rust
---

# TaxScheduleResult

Result of complete multi-tier tax calculation.

## Signature

```rust
pub struct TaxScheduleResult
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Result of complete multi-tier tax calculation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `net_total`
- `total_tax`
- `grand_total`
- `tax_lines`

## Source
Lines 42–51 in `crates/erp-trade/src/taxes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [taxes](/crates/erp-trade/src/taxes.md) |
