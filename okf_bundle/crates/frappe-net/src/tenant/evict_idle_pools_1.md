---
okf_version: "0.2"
type: Function
title: evict_idle_pools
description: Evicts idle pools exceeding the configured inactivity duration.
resource: crates/frappe-net/src/tenant.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:45:06Z"
concept_id: crates/frappe-net/src/tenant/evict_idle_pools_1
language: rust
---

# evict_idle_pools

Evicts idle pools exceeding the configured inactivity duration.

## Signature

```rust
pub fn evict_idle_pools(&self) -> usize
```

## Visibility

- `pub`

## Docstring

Evicts idle pools exceeding the configured inactivity duration.

## Source
Lines 130–138 in `crates/frappe-net/src/tenant.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [tenant](/crates/frappe-net/src/tenant.md) |
