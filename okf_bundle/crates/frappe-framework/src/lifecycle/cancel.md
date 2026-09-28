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
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/cancel
language: rust
---

# cancel

Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.

## Signature

```rust
impl DocumentController { pub fn cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
        script: Option<&str>,
    ) -> Result<(), DocumentError> }
```

## Visibility

- `pub`

## Docstring

Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.

## Source
Lines 190–215 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
