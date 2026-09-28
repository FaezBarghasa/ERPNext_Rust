---
okf_version: "0.2"
type: Function
title: authorize
description: Validates all cryptographic and physical isolation prerequisites before authorizing work commencement.
resource: crates/erp-asset/src/safety.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/safety/authorize
language: rust
---

# authorize

Validates all cryptographic and physical isolation prerequisites before authorizing work commencement.

## Signature

```rust
impl PermitToWork { pub fn authorize(&mut self, marshal_id: &str, secret_key: &str) -> Result<String, String> }
```

## Visibility

- `pub`

## Docstring

Validates all cryptographic and physical isolation prerequisites before authorizing work commencement.

## Source
Lines 62–98 in `crates/erp-asset/src/safety.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [safety](/crates/erp-asset/src/safety.md) |
