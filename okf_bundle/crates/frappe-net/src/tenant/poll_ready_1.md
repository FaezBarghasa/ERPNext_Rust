---
okf_version: "0.2"
type: Function
title: poll_ready
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/poll_ready_1
language: rust
---

# poll_ready

## Signature

```rust
fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>>
```

## Source
Lines 225–230 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
