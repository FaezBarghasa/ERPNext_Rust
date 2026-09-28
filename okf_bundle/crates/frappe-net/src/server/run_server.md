---
okf_version: "0.2"
type: Function
title: run_server
description: Runs the Actix Web Server on the specified address.
resource: crates/frappe-net/src/server.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-net/src/server/run_server
language: rust
---

# run_server

Runs the Actix Web Server on the specified address.

## Signature

```rust
pub fn run_server(addr: &str) -> std::io::Result<()>
```

## Visibility

- `pub`

## Docstring

Runs the Actix Web Server on the specified address.

## Source
Lines 30–38 in `crates/frappe-net/src/server.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [server](/crates/frappe-net/src/server.md) |
| calls | [configure_app](/crates/frappe-net/src/server/configure_app.md) |
| called_by | [main](/crates/rbench/src/main/main.md) |
