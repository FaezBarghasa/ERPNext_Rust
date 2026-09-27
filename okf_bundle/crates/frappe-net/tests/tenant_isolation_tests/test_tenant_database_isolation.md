---
okf_version: "0.2"
type: Function
title: test_tenant_database_isolation
description: "[tokio::test]"
resource: crates/frappe-net/tests/tenant_isolation_tests.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:04:10Z"
concept_id: crates/frappe-net/tests/tenant_isolation_tests/test_tenant_database_isolation
language: rust
---

# test_tenant_database_isolation

[tokio::test]

## Signature

```rust
fn test_tenant_database_isolation()
```

## Decorators

- `tokio::test`

## Docstring

[tokio::test]

## Source
Lines 5–57 in `crates/frappe-net/tests/tenant_isolation_tests.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant_isolation_tests](/crates/frappe-net/tests/tenant_isolation_tests.md) |
| calls | [TenantId](/crates/frappe-net/src/tenant/TenantId.md) |
| calls | [provision_tenant](/crates/frappe-net/src/tenant/provision_tenant.md) |
