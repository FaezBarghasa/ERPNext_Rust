---
okf_version: "0.2"
type: Function
title: surreal_type
description: Returns the corresponding SurrealQL field type string.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:15Z"
concept_id: crates/frappe-meta/src/schema/surreal_type
language: rust
---

# surreal_type

Returns the corresponding SurrealQL field type string.

## Signature

```rust
impl FieldType { pub fn surreal_type(&self) -> String }
```

## Visibility

- `pub`

## Docstring

Returns the corresponding SurrealQL field type string.
[must_use]

## Source
Lines 81–110 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
