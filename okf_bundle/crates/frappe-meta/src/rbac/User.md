---
okf_version: "0.2"
type: Class
title: User
description: User identity representation in the security graph.
resource: crates/frappe-meta/src/rbac.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:47:39Z"
concept_id: crates/frappe-meta/src/rbac/User
language: rust
---

# User

User identity representation in the security graph.

## Signature

```rust
pub struct User
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

User identity representation in the security graph.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `roles`
- `allowed_companies`

## Source
Lines 12–19 in `crates/frappe-meta/src/rbac.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [rbac](/crates/frappe-meta/src/rbac.md) |
