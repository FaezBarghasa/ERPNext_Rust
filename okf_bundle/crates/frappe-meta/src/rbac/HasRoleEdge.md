---
okf_version: "0.2"
type: Class
title: HasRoleEdge
description: "Graph edge definition: User -> Role"
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-meta/src/rbac/HasRoleEdge
language: rust
---

# HasRoleEdge

Graph edge definition: User -> Role

## Signature

```rust
pub struct HasRoleEdge
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

Graph edge definition: User -> Role
[derive(Debug, Clone, Serialize, Deserialize)]

## Methods

- `user_id`
- `role`

## Source
Lines 46–51 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
