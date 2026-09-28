---
okf_version: "0.2"
type: Function
title: call
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/call_1
language: rust
---

# call

## Signature

```rust
fn call(&self, req: ServiceRequest) -> Self::Future
```

## Source
Lines 232–244 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
| calls | [parse_tenant_id](/crates/frappe-net/src/tenant/parse_tenant_id.md) |
