---
okf_version: "0.2"
type: Function
title: filter_docs
description: "Pure kernel: case-sensitive substring filter over doc names."
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:23:38Z"
concept_id: crates/desk-components/src/signals/filter_docs
language: rust
---

# filter_docs

Pure kernel: case-sensitive substring filter over doc names.

## Signature

```rust
pub fn filter_docs(docs: &'a [String], query: &str) -> Vec<&'a String>
```

## Type Parameters

- `'a`

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Pure kernel: case-sensitive substring filter over doc names.
[must_use]

## Source
Lines 39–41 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
| called_by | [visible_from_signal](/crates/desk-components/src/signals/visible_from_signal.md) |
