---
okf_version: "0.2"
type: Function
title: spawn_maintenance_worker
description: Starts a periodic background worker for evicting idle tenant connections.
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:59:03Z"
concept_id: crates/frappe-net/src/tenant/spawn_maintenance_worker
language: rust
---

# spawn_maintenance_worker

Starts a periodic background worker for evicting idle tenant connections.

## Signature

```rust
impl ConnectionPoolManager { pub fn spawn_maintenance_worker(self: Arc<Self>, heartbeat: Duration) }
```

## Visibility

- `pub`

## Docstring

Starts a periodic background worker for evicting idle tenant connections.

## Source
Lines 136–144 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
