---
okf_version: "0.2"
type: Function
title: verify_integrity
description: Verifies tamper resistance against previous block in the chain.
resource: crates/frappe-storage/src/bitemporal.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/bitemporal/verify_integrity_1
language: rust
---

# verify_integrity

Verifies tamper resistance against previous block in the chain.

## Signature

```rust
pub fn verify_integrity(&self, expected_prev_hash: &str) -> bool
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Verifies tamper resistance against previous block in the chain.
[must_use]

## Source
Lines 93–95 in `crates/frappe-storage/src/bitemporal.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bitemporal](/crates/frappe-storage/src/bitemporal.md) |
