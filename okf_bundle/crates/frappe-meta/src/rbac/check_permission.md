---
okf_version: "0.2"
type: Function
title: check_permission
description: Evaluates whether a user with given roles is permitted to perform an operation.
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-meta/src/rbac/check_permission
language: rust
---

# check_permission

Evaluates whether a user with given roles is permitted to perform an operation.

## Signature

```rust
pub fn check_permission(
    user_roles: &[String],
    permission_edges: &[HasPermissionEdge],
    perm: Permission,
    target_permlevel: u8,
) -> bool
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Evaluates whether a user with given roles is permitted to perform an operation.
[must_use]

## Source
Lines 98–115 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
