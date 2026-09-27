---
okf_version: "0.2"
type: Class
title: LiveDiff
description: Live-query mutation diff pushed over WebSocket.
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:42Z"
concept_id: crates/frappe-storage/src/lib/LiveDiff
language: rust
---

# LiveDiff

Live-query mutation diff pushed over WebSocket.

## Signature

```rust
pub struct LiveDiff
```

## Decorators

- `derive(Debug, Clone, serde::Serialize, serde::Deserialize)`

## Visibility

- `pub`

## Docstring

Live-query mutation diff pushed over WebSocket.
[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

## Methods

- `table`
- `id`
- `op`
- `payload_json`

## Source
Lines 96–101 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
