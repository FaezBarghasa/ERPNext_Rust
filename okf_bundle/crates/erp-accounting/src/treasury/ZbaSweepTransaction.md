---
okf_version: "0.2"
type: Class
title: ZbaSweepTransaction
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-accounting/src/treasury.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/treasury/ZbaSweepTransaction
language: rust
---

# ZbaSweepTransaction

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ZbaSweepTransaction
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `sweep_id`
- `from_account_id`
- `to_account_id`
- `sweep_amount`
- `intercompany_loan_ref`
- `annual_interest_rate`

## Source
Lines 15–22 in `crates/erp-accounting/src/treasury.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [treasury](/crates/erp-accounting/src/treasury.md) |
