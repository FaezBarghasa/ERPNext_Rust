---
okf_version: "0.2"
type: Function
title: open_tenant
description: "Open an in-memory instance, create NS+DB, and return the handle."
resource: crates/frappe-storage/src/surreal.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:28:21Z"
concept_id: crates/frappe-storage/src/surreal/open_tenant
language: rust
---

# open_tenant

Open an in-memory instance, create NS+DB, and return the handle.

## Signature

```rust
pub fn open_tenant(tenant_ns: &str, db: &str) -> surrealdb::Result<Surreal<surrealdb::engine::local::Db>>
```

## Visibility

- `pub`

## Docstring

Open an in-memory instance, create NS+DB, and return the handle.

## Source
Lines 5–9 in `crates/frappe-storage/src/surreal.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [surreal](/crates/frappe-storage/src/surreal.md) |
| called_by | [ns_db_roundtrip](/crates/frappe-storage/src/surreal/ns_db_roundtrip.md) |
