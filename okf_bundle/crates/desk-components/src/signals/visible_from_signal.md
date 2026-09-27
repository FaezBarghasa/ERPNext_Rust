---
okf_version: "0.2"
type: Function
title: visible_from_signal
description: "Component-side helper: read a doc-list signal through the reactive kernel."
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:44:12Z"
concept_id: crates/desk-components/src/signals/visible_from_signal
language: rust
---

# visible_from_signal

Component-side helper: read a doc-list signal through the reactive kernel.

## Signature

```rust
pub fn visible_from_signal(sig: &Signal<Vec<String>>, query: &str) -> Vec<String>
```

## Visibility

- `pub`

## Docstring

Component-side helper: read a doc-list signal through the reactive kernel.

## Source
Lines 12–14 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
| calls | [filter_docs](/crates/desk-components/src/signals/filter_docs.md) |
