---
okf_version: "0.2"
type: Function
title: submit
description: "Submits a draft document, mutating docstatus to 1 and triggering submission hooks."
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:57:18Z"
concept_id: crates/frappe-framework/src/lifecycle/submit_1
language: rust
---

# submit

Submits a draft document, mutating docstatus to 1 and triggering submission hooks.

## Signature

```rust
pub fn submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        script: Option<&str>,
    ) -> Result<(), DocumentError>
```

## Visibility

- `pub`

## Docstring

Submits a draft document, mutating docstatus to 1 and triggering submission hooks.

## Source
Lines 167–190 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
