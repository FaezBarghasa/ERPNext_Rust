---
okf_version: "0.2"
type: Function
title: generate_token
description: "Generates a signed token string: `hex(hmac(media_id || expiry))`."
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:00:25Z"
concept_id: crates/erp-cms/src/security/generate_token
language: rust
---

# generate_token

Generates a signed token string: `hex(hmac(media_id || expiry))`.

## Signature

```rust
impl HmacStreamingSigner { pub fn generate_token(&self, media_id: &str, expiry_timestamp: u64) -> String }
```

## Visibility

- `pub`

## Docstring

Generates a signed token string: `hex(hmac(media_id || expiry))`.

## Source
Lines 101–108 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
| calls | [encode](/crates/erp-cms/src/security/encode.md) |
