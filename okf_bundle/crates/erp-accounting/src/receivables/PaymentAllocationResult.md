---
okf_version: "0.2"
type: Class
title: PaymentAllocationResult
description: Result of an individual invoice payment allocation.
resource: crates/erp-accounting/src/receivables.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:10:14Z"
concept_id: crates/erp-accounting/src/receivables/PaymentAllocationResult
language: rust
---

# PaymentAllocationResult

Result of an individual invoice payment allocation.

## Signature

```rust
pub struct PaymentAllocationResult
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Result of an individual invoice payment allocation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `voucher_no`
- `allocated_amount`
- `remaining_outstanding`

## Source
Lines 21–28 in `crates/erp-accounting/src/receivables.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [receivables](/crates/erp-accounting/src/receivables.md) |
