---
okf_version: "0.2"
type: Function
title: encode
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:25:54Z"
concept_id: crates/erp-cms/src/security/encode
language: rust
---

# encode

## Signature

```rust
pub fn encode(data: impl AsRef<[u8]>) -> String
```

## Visibility

- `pub`

## Source
Lines 188–193 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
| called_by | [generate_token](/crates/erp-cms/src/security/generate_token.md) |
