---
okf_version: "0.2"
type: Function
title: convert_to_sales_order
description: "Converts an approved Quotation into a confirmed SalesOrder, preserving locked pricing."
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:56:26Z"
concept_id: crates/erp-crm/src/pipeline/convert_to_sales_order
language: rust
---

# convert_to_sales_order

Converts an approved Quotation into a confirmed SalesOrder, preserving locked pricing.

## Signature

```rust
impl CrmPipeline { pub fn convert_to_sales_order(
        quotation: &mut Quotation,
        as_of_date: NaiveDate,
        sales_order_id: &str,
    ) -> Result<SalesOrder, CrmError> }
```

## Visibility

- `pub`

## Docstring

Converts an approved Quotation into a confirmed SalesOrder, preserving locked pricing.

## Source
Lines 118–146 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
