---
okf_version: "0.2"
type: Function
title: list_resource
description: "Generic handler for listing documents: `GET /api/v1/resource/{doctype}`"
resource: crates/frappe-net/src/routes.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:06:13Z"
concept_id: crates/frappe-net/src/routes/list_resource
language: rust
---

# list_resource

Generic handler for listing documents: `GET /api/v1/resource/{doctype}`

## Signature

```rust
pub fn list_resource(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<ListQuery>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder
```

## Visibility

- `pub`

## Docstring

Generic handler for listing documents: `GET /api/v1/resource/{doctype}`

## Source
Lines 15–44 in `crates/frappe-net/src/routes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [routes](/crates/frappe-net/src/routes.md) |
