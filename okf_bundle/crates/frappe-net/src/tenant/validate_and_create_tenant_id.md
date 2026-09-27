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
concept_id: crates/frappe-net/src/tenant/validate_and_create_tenant_id
language: rust
---

# validate_and_create_tenant_id

## Signature

```rust
fn validate_and_create_tenant_id(tenant_str: &str) -> Result<TenantId, TenantError>
```

## Source
Lines 167–176 in `crates/frappe-net/src/tenant.rs`
