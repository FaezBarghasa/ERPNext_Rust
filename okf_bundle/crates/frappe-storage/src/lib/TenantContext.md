---
okf_version: "0.2"
type: Class
title: TenantContext
description: Contextual tenant information attached to requests.
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:42Z"
concept_id: crates/frappe-storage/src/lib/TenantContext
language: rust
---

# TenantContext

Contextual tenant information attached to requests.

## Signature

```rust
pub struct TenantContext
```

## Decorators

- `derive(Debug, Clone)`

## Visibility

- `pub`

## Docstring

Contextual tenant information attached to requests.
[derive(Debug, Clone)]

## Methods

- `tenant_id`
- `site_name`
- `user_id`
- `roles`

## Source
Lines 13–18 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
