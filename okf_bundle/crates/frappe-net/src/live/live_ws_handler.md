---
okf_version: "0.2"
type: Function
title: live_ws_handler
description: "WebSocket Live Query Streaming Actor (Milestone 1.8 & 4.3)."
resource: crates/frappe-net/src/live.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-net"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:01:51Z"
concept_id: crates/frappe-net/src/live/live_ws_handler
language: rust
---

# live_ws_handler

WebSocket Live Query Streaming Actor (Milestone 1.8 & 4.3).

## Signature

```rust
pub fn live_ws_handler(
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<HttpResponse>
```

## Visibility

- `pub`

## Docstring

WebSocket Live Query Streaming Actor (Milestone 1.8 & 4.3).

## Source
Lines 5–22 in `crates/frappe-net/src/live.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [live](/crates/frappe-net/src/live.md) |
