---
okf_version: "0.2"
type: Function
title: validate_and_create_tenant_id
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/validate_and_create_tenant_id
language: rust
---

# validate_and_create_tenant_id

## Signature

```rust
fn validate_and_create_tenant_id(tenant_str: &str) -> Result<TenantId, TenantError>
```

## Source
Lines 175–187 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
| calls | [TenantId](/crates/frappe-net/src/tenant/TenantId.md) |
| called_by | [parse_tenant_id](/crates/frappe-net/src/tenant/parse_tenant_id.md) |
| called_by | [provision_tenant](/crates/frappe-net/src/tenant/provision_tenant.md) |
