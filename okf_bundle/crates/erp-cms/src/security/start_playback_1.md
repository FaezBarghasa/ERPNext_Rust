---
okf_version: "0.2"
type: Function
title: start_playback
description: Starts a playback session if subscription is active and concurrency limit is not breached.
resource: crates/erp-cms/src/security.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:25:54Z"
concept_id: crates/erp-cms/src/security/start_playback_1
language: rust
---

# start_playback

Starts a playback session if subscription is active and concurrency limit is not breached.

## Signature

```rust
pub fn start_playback(
        &self,
        user_id: &str,
        session_id: String,
        is_subscription_active: bool,
        max_concurrent_streams: usize,
    ) -> Result<(), StreamingError>
```

## Visibility

- `pub`

## Docstring

Starts a playback session if subscription is active and concurrency limit is not breached.

## Source
Lines 150–175 in `crates/erp-cms/src/security.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [security](/crates/erp-cms/src/security.md) |
