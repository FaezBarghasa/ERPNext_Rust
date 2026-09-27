---
okf_version: "0.2"
type: Function
title: live_query
description: Generates a SurrealQL LIVE SELECT registration query.
resource: crates/frappe-net/src/live.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:01:51Z"
concept_id: crates/frappe-net/src/live/live_query
language: rust
---

# live_query

Generates a SurrealQL LIVE SELECT registration query.

## Signature

```rust
pub fn live_query(tenant_ns: &str, table: &str, id: &str) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates a SurrealQL LIVE SELECT registration query.
[must_use]

## Source
Lines 26–31 in `crates/frappe-net/src/live.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [live](/crates/frappe-net/src/live.md) |
| called_by | [test_live_query_format](/crates/frappe-net/src/live/test_live_query_format.md) |
