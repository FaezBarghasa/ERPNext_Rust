---
okf_version: "0.2"
type: Class
title: VideoHudState
description: SVoD Video HUD Controller State (Milestone 5.7).
resource: crates/desk-components/src/video_hud.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:03:36Z"
concept_id: crates/desk-components/src/video_hud/VideoHudState
language: rust
---

# VideoHudState

SVoD Video HUD Controller State (Milestone 5.7).

## Signature

```rust
pub struct VideoHudState
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq)`

## Visibility

- `pub`

## Docstring

SVoD Video HUD Controller State (Milestone 5.7).
[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

## Methods

- `media_id`
- `current_time_secs`
- `duration_secs`
- `is_playing`
- `chapters`
- `comments`

## Source
Lines 12–19 in `crates/desk-components/src/video_hud.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [video_hud](/crates/desk-components/src/video_hud.md) |
