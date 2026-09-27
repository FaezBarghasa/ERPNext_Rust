---
okf_version: "0.2"
type: Function
title: push_doc
description: Helper appending a new doc name to a Signal list.
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:23:38Z"
concept_id: crates/desk-components/src/signals/push_doc
language: rust
---

# push_doc

Helper appending a new doc name to a Signal list.

## Signature

```rust
pub fn push_doc(sig: &Signal<Vec<String>>, name: String)
```

## Visibility

- `pub`

## Docstring

Helper appending a new doc name to a Signal list.

## Source
Lines 50–52 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
| called_by | [test_signal_reactivity_and_filtering](/crates/desk-components/src/signals/test_signal_reactivity_and_filtering.md) |
