---
okf_version: "0.2"
type: Function
title: parse_byte_range
description: HTTP 206 Byte Range Parser (Milestone 5.4).
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:25:54Z"
concept_id: crates/erp-cms/src/security/parse_byte_range
language: rust
---

# parse_byte_range

HTTP 206 Byte Range Parser (Milestone 5.4).

## Signature

```rust
pub fn parse_byte_range(range_header: &str, total_size: u64) -> Result<(u64, u64), StreamingError>
```

## Visibility

- `pub`

## Docstring

HTTP 206 Byte Range Parser (Milestone 5.4).

## Source
Lines 37–85 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
| called_by | [test_byte_range_parser](/crates/erp-cms/src/lib/test_byte_range_parser.md) |
