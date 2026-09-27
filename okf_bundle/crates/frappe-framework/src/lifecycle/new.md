---
okf_version: "0.2"
type: Function
title: new
description: Creates a new draft document instance.
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:57:18Z"
concept_id: crates/frappe-framework/src/lifecycle/new
language: rust
---

# new

Creates a new draft document instance.

## Signature

```rust
impl Document { pub fn new(doctype: &str, data: serde_json::Value) -> Self }
```

## Visibility

- `pub`

## Docstring

Creates a new draft document instance.
[must_use]

## Source
Lines 45–52 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
