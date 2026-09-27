---
okf_version: "0.2"
type: Function
title: generate_variant_playlist
description: Generates an individual variant segment index playlist.
resource: crates/erp-cms/src/transcoder.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:55:24Z"
concept_id: crates/erp-cms/src/transcoder/generate_variant_playlist
language: rust
---

# generate_variant_playlist

Generates an individual variant segment index playlist.

## Signature

```rust
impl HlsPlaylistGenerator { pub fn generate_variant_playlist(variant_name: &str, segment_count: usize, segment_duration_secs: u32) -> String }
```

## Visibility

- `pub`

## Docstring

Generates an individual variant segment index playlist.
[must_use]

## Source
Lines 59–74 in `crates/erp-cms/src/transcoder.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [transcoder](/crates/erp-cms/src/transcoder.md) |
