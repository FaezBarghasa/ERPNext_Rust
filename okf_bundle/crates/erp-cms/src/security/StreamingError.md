---
okf_version: "0.2"
type: Class
title: StreamingError
description: Streaming and CMS security errors.
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:00:25Z"
concept_id: crates/erp-cms/src/security/StreamingError
language: rust
---

# StreamingError

Streaming and CMS security errors.

## Signature

```rust
pub enum StreamingError
```

## Decorators

- `derive(Debug, Error, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Streaming and CMS security errors.
[derive(Debug, Error, PartialEq, Eq)]

## Methods

- `start`
- `end`
- `total`
- `expiry_timestamp`
- `current_timestamp`
- `max_streams`

## Source
Lines 11–33 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
