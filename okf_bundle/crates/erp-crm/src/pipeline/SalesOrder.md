---
okf_version: "0.2"
type: Class
title: SalesOrder
description: Confirmed Sales Order.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:34:58Z"
concept_id: crates/erp-crm/src/pipeline/SalesOrder
language: rust
---

# SalesOrder

Confirmed Sales Order.

## Signature

```rust
pub struct SalesOrder
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Confirmed Sales Order.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `customer`
- `quotation_ref`
- `items`
- `net_total`
- `posting_date`

## Source
Lines 106–113 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
