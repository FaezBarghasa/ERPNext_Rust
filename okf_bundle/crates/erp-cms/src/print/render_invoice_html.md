---
okf_version: "0.2"
type: Function
title: render_invoice_html
description: Lightweight HTML invoice template renderer.
resource: crates/erp-cms/src/print.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:56:35Z"
concept_id: crates/erp-cms/src/print/render_invoice_html
language: rust
---

# render_invoice_html

Lightweight HTML invoice template renderer.

## Signature

```rust
pub fn render_invoice_html(invoice_no: &str, total_amount: &str, currency: &str) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Lightweight HTML invoice template renderer.
[must_use]

## Source
Lines 3–31 in `crates/erp-cms/src/print.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [print](/crates/erp-cms/src/print.md) |
| called_by | [test_render_invoice_html](/crates/erp-cms/src/print/test_render_invoice_html.md) |
