---
okf_version: "0.2"
type: Function
title: new
description: Creates a new connection pool manager.
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/new
language: rust
---

# new

Creates a new connection pool manager.

## Signature

```rust
impl ConnectionPoolManager { pub fn new(inactivity_threshold: Duration) -> Self }
```

## Visibility

- `pub`

## Docstring

Creates a new connection pool manager.
[must_use]

## Source
Lines 89–94 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
