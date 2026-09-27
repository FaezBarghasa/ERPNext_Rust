---
okf_version: "0.2"
type: Function
title: get_or_initialize_client
description: "Retrieves an active connection handle for the tenant, initializing if missing."
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:59:03Z"
concept_id: crates/frappe-net/src/tenant/get_or_initialize_client_1
language: rust
---

# get_or_initialize_client

Retrieves an active connection handle for the tenant, initializing if missing.

## Signature

```rust
pub fn get_or_initialize_client(
        &self,
        tenant: &TenantId,
    ) -> Result<Surreal<surrealdb::engine::local::Db>, TenantError>
```

## Visibility

- `pub`

## Docstring

Retrieves an active connection handle for the tenant, initializing if missing.

## Source
Lines 92–122 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
