---
okf_version: "0.2"
type: Class
title: EInvoiceDocument
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-trade/src/einvoice.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:27:37Z"
concept_id: crates/erp-trade/src/einvoice/EInvoiceDocument
language: rust
---

# EInvoiceDocument

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct EInvoiceDocument
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `invoice_id`
- `standard`
- `seller_tax_id`
- `buyer_tax_id`
- `issue_date`
- `total_taxable_amount`
- `total_vat_amount`
- `payable_amount`
- `currency`

## Source
Lines 15–25 in `crates/erp-trade/src/einvoice.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [einvoice](/crates/erp-trade/src/einvoice.md) |
