---
okf_version: "0.2"
type: Function
title: parse_tenant_id
description: Resolves tenant identity from request headers or host string.
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:59:03Z"
concept_id: crates/frappe-net/src/tenant/parse_tenant_id
language: rust
---

# parse_tenant_id

Resolves tenant identity from request headers or host string.

## Signature

```rust
pub fn parse_tenant_id(headers: &actix_web::http::header::HeaderMap, host: &str) -> Result<TenantId, TenantError>
```

## Visibility

- `pub`

## Docstring

Resolves tenant identity from request headers or host string.

## Source
Lines 148–165 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
| calls | [validate_and_create_tenant_id](/crates/frappe-net/src/tenant/validate_and_create_tenant_id.md) |
| called_by | [call](/crates/frappe-net/src/tenant/call.md) |
