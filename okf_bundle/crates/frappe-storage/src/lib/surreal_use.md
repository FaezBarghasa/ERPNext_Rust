---
okf_version: "0.2"
type: Function
title: surreal_use
description: Generates SurrealQL USE statement.
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:42Z"
concept_id: crates/frappe-storage/src/lib/surreal_use
language: rust
---

# surreal_use

Generates SurrealQL USE statement.

## Signature

```rust
impl TenantContext { pub fn surreal_use(&self) -> String }
```

## Visibility

- `pub`

## Docstring

Generates SurrealQL USE statement.
[must_use]

## Source
Lines 23–25 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
