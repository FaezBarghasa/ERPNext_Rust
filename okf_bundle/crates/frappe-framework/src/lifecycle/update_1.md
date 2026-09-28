---
okf_version: "0.2"
type: Function
title: update
description: Mutates an existing document while rejecting edits to submitted documents.
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/update_1
language: rust
---

# update

Mutates an existing document while rejecting edits to submitted documents.

## Signature

```rust
pub fn update(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
        new_data: serde_json::Value,
        script: Option<&str>,
    ) -> Result<(), DocumentError>
```

## Visibility

- `pub`

## Docstring

Mutates an existing document while rejecting edits to submitted documents.

## Source
Lines 134–161 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
