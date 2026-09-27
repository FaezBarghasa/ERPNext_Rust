---
okf_version: "0.2"
type: Function
title: compile_rls_policy
description: Generates SurrealDB Row-Level Security (RLS) predicate for a table.
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:47:39Z"
concept_id: crates/frappe-meta/src/rbac/compile_rls_policy
language: rust
---

# compile_rls_policy

Generates SurrealDB Row-Level Security (RLS) predicate for a table.

## Signature

```rust
pub fn compile_rls_policy(doctype: &str, roles_with_read: &[String], roles_with_write: &[String]) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates SurrealDB Row-Level Security (RLS) predicate for a table.
[must_use]

## Source
Lines 118–136 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
