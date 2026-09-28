---
okf_version: "0.2"
type: Function
title: insert
description: "Handles document insertion: runs naming series, validation hooks, and commits to draft state."
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/insert_1
language: rust
---

# insert

Handles document insertion: runs naming series, validation hooks, and commits to draft state.

## Signature

```rust
pub fn insert(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        script: Option<&str>,
        year: u32,
        seq: u64,
    ) -> Result<(), DocumentError>
```

## Visibility

- `pub`

## Docstring

Handles document insertion: runs naming series, validation hooks, and commits to draft state.

## Source
Lines 94–131 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
