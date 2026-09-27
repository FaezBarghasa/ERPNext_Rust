---
okf_version: "0.2"
type: Function
title: update
description: Mutates the signal value in-place with a closure.
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:23:38Z"
concept_id: crates/desk-components/src/signals/update
language: rust
---

# update

Mutates the signal value in-place with a closure.

## Signature

```rust
impl Signal<T> { pub fn update(&self, f: impl FnOnce(&mut T) -> R) -> R }
```

## Type Parameters

- `R`

## Visibility

- `pub`

## Docstring

Mutates the signal value in-place with a closure.

## Source
Lines 31–34 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
