---
okf_version: "0.2"
type: Function
title: generate_ubl_xml
description: Generates structured XML payload compliant with targeted statutory e-invoicing standard.
resource: crates/erp-trade/src/einvoice.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:27:37Z"
concept_id: crates/erp-trade/src/einvoice/generate_ubl_xml_1
language: rust
---

# generate_ubl_xml

Generates structured XML payload compliant with targeted statutory e-invoicing standard.

## Signature

```rust
pub fn generate_ubl_xml(doc: &EInvoiceDocument) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates structured XML payload compliant with targeted statutory e-invoicing standard.
[must_use]

## Source
Lines 32–74 in `crates/erp-trade/src/einvoice.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [einvoice](/crates/erp-trade/src/einvoice.md) |
