---
okf_version: "0.2"
type: Function
title: configure_app
description: Configures the Actix Web ERP API application.
resource: crates/frappe-net/src/server.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-net/src/server/configure_app
language: rust
---

# configure_app

Configures the Actix Web ERP API application.

## Signature

```rust
pub fn configure_app(cfg: &mut web::ServiceConfig, pool_mgr: ConnectionPoolManager)
```

## Visibility

- `pub`

## Docstring

Configures the Actix Web ERP API application.

## Source
Lines 15–27 in `crates/frappe-net/src/server.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [server](/crates/frappe-net/src/server.md) |
| called_by | [run_server](/crates/frappe-net/src/server/run_server.md) |
