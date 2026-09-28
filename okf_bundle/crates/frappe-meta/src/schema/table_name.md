---
okf_version: "0.2"
type: Function
title: table_name
description: Returns the database table name derived from the DocType name.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:40:46Z"
concept_id: crates/frappe-meta/src/schema/table_name
language: rust
---

# table_name

Returns the database table name derived from the DocType name.

## Signature

```rust
impl DocTypeSchema { pub fn table_name(&self) -> String }
```

## Visibility

- `pub`

## Docstring

Returns the database table name derived from the DocType name.
[must_use]

## Source
Lines 276–278 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
