---
okf_version: "0.2"
type: Function
title: get_resource
description: "Generic handler for getting a document by ID: `GET /api/v1/resource/{doctype}/{id}`"
resource: crates/frappe-net/src/routes.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:06:13Z"
concept_id: crates/frappe-net/src/routes/get_resource
language: rust
---

# get_resource

Generic handler for getting a document by ID: `GET /api/v1/resource/{doctype}/{id}`

## Signature

```rust
pub fn get_resource(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    pool_mgr: web::Data<ConnectionPoolManager>,
) -> impl Responder
```

## Visibility

- `pub`

## Docstring

Generic handler for getting a document by ID: `GET /api/v1/resource/{doctype}/{id}`

## Source
Lines 47–75 in `crates/frappe-net/src/routes.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [routes](/crates/frappe-net/src/routes.md) |
