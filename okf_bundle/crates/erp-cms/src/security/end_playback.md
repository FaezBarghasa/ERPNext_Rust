---
okf_version: "0.2"
type: Function
title: end_playback
description: Ends an active playback session.
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:00:25Z"
concept_id: crates/erp-cms/src/security/end_playback
language: rust
---

# end_playback

Ends an active playback session.

## Signature

```rust
impl SvodPlaybackManager { pub fn end_playback(&self, user_id: &str, session_id: &str) }
```

## Visibility

- `pub`

## Docstring

Ends an active playback session.

## Source
Lines 177–183 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
