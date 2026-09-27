---
okf_version: "0.2"
type: Function
title: visible_from_signal
description: Helper reading filtered document list from a Signal.
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:23:38Z"
concept_id: crates/desk-components/src/signals/visible_from_signal
language: rust
---

# visible_from_signal

Helper reading filtered document list from a Signal.

## Signature

```rust
pub fn visible_from_signal(sig: &Signal<Vec<String>>, query: &str) -> Vec<String>
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Helper reading filtered document list from a Signal.
[must_use]

## Source
Lines 45–47 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
| calls | [filter_docs](/crates/desk-components/src/signals/filter_docs.md) |
| called_by | [test_signal_reactivity_and_filtering](/crates/desk-components/src/signals/test_signal_reactivity_and_filtering.md) |
