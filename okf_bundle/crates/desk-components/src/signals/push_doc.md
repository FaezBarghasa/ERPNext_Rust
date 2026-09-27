---
okf_version: "0.2"
type: Function
title: push_doc
description: "Component-side helper: push a newly created doc into the list signal."
resource: crates/desk-components/src/signals.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:44:12Z"
concept_id: crates/desk-components/src/signals/push_doc
language: rust
---

# push_doc

Component-side helper: push a newly created doc into the list signal.

## Signature

```rust
pub fn push_doc(sig: &mut Signal<Vec<String>>, name: String)
```

## Visibility

- `pub`

## Docstring

Component-side helper: push a newly created doc into the list signal.

## Source
Lines 17–19 in `crates/desk-components/src/signals.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [signals](/crates/desk-components/src/signals.md) |
