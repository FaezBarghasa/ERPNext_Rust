---
okf_version: "0.2"
type: Class
title: TenantId
description: Unique Tenant Identifier.
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/TenantId
language: rust
---

# TenantId

Unique Tenant Identifier.

## Signature

```rust
pub struct TenantId
```

## Decorators

- `derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

Unique Tenant Identifier.
[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]

## Source
Lines 19–19 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
| called_by | [validate_and_create_tenant_id](/crates/frappe-net/src/tenant/validate_and_create_tenant_id.md) |
| called_by | [test_tenant_database_isolation](/crates/frappe-net/tests/tenant_isolation_tests/test_tenant_database_isolation.md) |
