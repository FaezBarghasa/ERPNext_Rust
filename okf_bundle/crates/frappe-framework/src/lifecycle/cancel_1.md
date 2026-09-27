---
okf_version: "0.2"
type: Function
title: cancel
description: "Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks."
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:57:18Z"
concept_id: crates/frappe-framework/src/lifecycle/cancel_1
language: rust
---

# cancel

Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.

## Signature

```rust
pub fn cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
        script: Option<&str>,
    ) -> Result<(), DocumentError>
```

## Visibility

- `pub`

## Docstring

Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.

## Source
Lines 193–216 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
