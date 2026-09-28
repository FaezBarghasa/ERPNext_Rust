---
okf_version: "0.2"
type: Class
title: TaxLineResult
description: Individual computed tax line output.
resource: crates/erp-trade/src/taxes.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-trade/src/taxes/TaxLineResult
language: rust
---

# TaxLineResult

Individual computed tax line output.

## Signature

```rust
pub struct TaxLineResult
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Individual computed tax line output.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `account_head`
- `rate`
- `tax_amount`
- `total`

## Source
Lines 29–38 in `crates/erp-trade/src/taxes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [taxes](/crates/erp-trade/src/taxes.md) |
