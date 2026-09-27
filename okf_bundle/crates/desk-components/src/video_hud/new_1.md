---
okf_version: "0.2"
type: Function
title: new
description: Creates a new Video HUD state.
resource: crates/desk-components/src/video_hud.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:03:36Z"
concept_id: crates/desk-components/src/video_hud/new_1
language: rust
---

# new

Creates a new Video HUD state.

## Signature

```rust
pub fn new(media_id: &str, duration_secs: f64, chapters: Vec<ChapterMarker>) -> Self
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Creates a new Video HUD state.
[must_use]

## Source
Lines 24–33 in `crates/desk-components/src/video_hud.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [video_hud](/crates/desk-components/src/video_hud.md) |
