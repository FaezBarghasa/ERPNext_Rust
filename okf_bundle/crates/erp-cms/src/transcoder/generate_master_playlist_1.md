---
okf_version: "0.2"
type: Function
title: generate_master_playlist
description: Generates valid master.m3u8 playlist indexing all variants.
resource: crates/erp-cms/src/transcoder.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:55:24Z"
concept_id: crates/erp-cms/src/transcoder/generate_master_playlist_1
language: rust
---

# generate_master_playlist

Generates valid master.m3u8 playlist indexing all variants.

## Signature

```rust
pub fn generate_master_playlist(variants: &[VideoVariant]) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Generates valid master.m3u8 playlist indexing all variants.
[must_use]

## Source
Lines 40–55 in `crates/erp-cms/src/transcoder.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [transcoder](/crates/erp-cms/src/transcoder.md) |
