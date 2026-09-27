---
okf_version: "0.2"
type: Function
title: verify_token
description: Validates an incoming signed token against media ID and current timestamp.
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:25:54Z"
concept_id: crates/erp-cms/src/security/verify_token_1
language: rust
---

# verify_token

Validates an incoming signed token against media ID and current timestamp.

## Signature

```rust
pub fn verify_token(
        &self,
        media_id: &str,
        expiry_timestamp: u64,
        current_timestamp: u64,
        provided_token: &str,
    ) -> Result<(), StreamingError>
```

## Visibility

- `pub`

## Docstring

Validates an incoming signed token against media ID and current timestamp.

## Source
Lines 112–132 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
