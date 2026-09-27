---
okf_version: "0.2"
type: Function
title: relate
description: Helper function to generate SurrealQL RELATE graph edge statement.
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:42Z"
concept_id: crates/frappe-storage/src/lib/relate
language: rust
---

# relate

Helper function to generate SurrealQL RELATE graph edge statement.

## Signature

```rust
pub fn relate(a: &str, verb: &str, b: &str, attrs: &str) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Helper function to generate SurrealQL RELATE graph edge statement.
[must_use]

## Source
Lines 112–118 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
