---
okf_version: "0.2"
type: Function
title: provision_tenant
description: Cloud Multi-Tenant Provisioning Coordinator (Milestone 5.8).
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/provision_tenant
language: rust
---

# provision_tenant

Cloud Multi-Tenant Provisioning Coordinator (Milestone 5.8).

## Signature

```rust
pub fn provision_tenant(
    pool_manager: &ConnectionPoolManager,
    tenant_id: &str,
) -> Result<Surreal<surrealdb::engine::local::Db>, TenantError>
```

## Visibility

- `pub`

## Docstring

Cloud Multi-Tenant Provisioning Coordinator (Milestone 5.8).

## Source
Lines 248–272 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
| calls | [validate_and_create_tenant_id](/crates/frappe-net/src/tenant/validate_and_create_tenant_id.md) |
| called_by | [test_tenant_database_isolation](/crates/frappe-net/tests/tenant_isolation_tests/test_tenant_database_isolation.md) |
