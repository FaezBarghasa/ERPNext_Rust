---
okf_version: "0.2"
type: Function
title: peek
description: Reads the current signal value without tracking.
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:23:38Z"
concept_id: crates/desk-components/src/signals/peek_1
language: rust
---

# peek

Reads the current signal value without tracking.

## Signature

```rust
pub fn peek(&self, f: impl FnOnce(&T) -> R) -> R
```

## Type Parameters

- `R`

## Visibility

- `pub`

## Docstring

Reads the current signal value without tracking.

## Source
Lines 19–22 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
