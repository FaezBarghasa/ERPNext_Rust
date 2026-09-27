---
okf_version: "0.2"
type: Function
title: allows
description: Checks if this edge grants the specified permission.
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:47:39Z"
concept_id: crates/frappe-meta/src/rbac/allows_1
language: rust
---

# allows

Checks if this edge grants the specified permission.

## Signature

```rust
pub fn allows(&self, perm: Permission) -> bool
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Checks if this edge grants the specified permission.
[must_use]

## Source
Lines 81–93 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
