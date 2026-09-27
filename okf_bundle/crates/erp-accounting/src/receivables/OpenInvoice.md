---
okf_version: "0.2"
type: Class
title: OpenInvoice
description: Unsettled customer sales invoice for payment allocation.
resource: crates/erp-accounting/src/receivables.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:10:14Z"
concept_id: crates/erp-accounting/src/receivables/OpenInvoice
language: rust
---

# OpenInvoice

Unsettled customer sales invoice for payment allocation.

## Signature

```rust
pub struct OpenInvoice
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Unsettled customer sales invoice for payment allocation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `voucher_no`
- `posting_date`
- `grand_total`
- `outstanding_amount`

## Source
Lines 8–17 in `crates/erp-accounting/src/receivables.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [receivables](/crates/erp-accounting/src/receivables.md) |
