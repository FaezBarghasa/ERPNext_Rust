---
okf_version: "0.2"
type: Function
title: main
description: "[tokio::main]"
resource: crates/rbench/src/main.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:rbench"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T22:03:08Z"
concept_id: crates/rbench/src/main/main
language: rust
---

# main

[tokio::main]

## Signature

```rust
fn main() -> Result<(), Box<dyn std::error::Error>>
```

## Decorators

- `tokio::main`

## Docstring

[tokio::main]

## Source
Lines 8–153 in `crates/rbench/src/main.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [src](/crates/rbench/src/main.md) |
| calls | [open_tenant](/crates/frappe-storage/src/surreal/open_tenant.md) |
| calls | [compile_to_surrealql](/crates/frappe-meta/src/schema_compiler/compile_to_surrealql.md) |
| calls | [run_server](/crates/frappe-net/src/server/run_server.md) |
