---
okf_version: "0.2"
type: Function
title: new
description: Creates a new signer with the specified secret key.
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:00:25Z"
concept_id: crates/erp-cms/src/security/new
language: rust
---

# new

Creates a new signer with the specified secret key.

## Signature

```rust
impl HmacStreamingSigner { pub fn new(secret_key: &[u8]) -> Self }
```

## Visibility

- `pub`

## Docstring

Creates a new signer with the specified secret key.
[must_use]

## Source
Lines 94–98 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
