---
okf_version: "0.2"
type: Class
title: HasPermissionEdge
description: "Graph edge definition: Role -> DocType with granular permissions."
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:47:39Z"
concept_id: crates/frappe-meta/src/rbac/HasPermissionEdge
language: rust
---

# HasPermissionEdge

Graph edge definition: Role -> DocType with granular permissions.

## Signature

```rust
pub struct HasPermissionEdge
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

Graph edge definition: Role -> DocType with granular permissions.
[derive(Debug, Clone, Serialize, Deserialize)]

## Methods

- `role`
- `doctype`
- `p_read`
- `p_write`
- `p_create`
- `p_delete`
- `p_submit`
- `p_cancel`
- `p_amend`
- `permlevel`

## Source
Lines 55–76 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
