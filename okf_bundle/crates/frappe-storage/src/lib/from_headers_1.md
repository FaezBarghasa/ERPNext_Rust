---
okf_version: "0.2"
type: Function
title: from_headers
description: Resolves tenant context from HTTP headers.
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/lib/from_headers_1
language: rust
---

# from_headers

Resolves tenant context from HTTP headers.

## Signature

```rust
pub fn from_headers(headers: &HashMap<String, String>, host: &str) -> Option<Self>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Resolves tenant context from HTTP headers.
[must_use]

## Source
Lines 32–44 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
