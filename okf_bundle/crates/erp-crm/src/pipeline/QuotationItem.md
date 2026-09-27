---
okf_version: "0.2"
type: Class
title: QuotationItem
description: Item line in a sales quotation or order.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:34:58Z"
concept_id: crates/erp-crm/src/pipeline/QuotationItem
language: rust
---

# QuotationItem

Item line in a sales quotation or order.

## Signature

```rust
pub struct QuotationItem
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Item line in a sales quotation or order.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `item_code`
- `qty`
- `rate`
- `amount`

## Source
Lines 77–82 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
