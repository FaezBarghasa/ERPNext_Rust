---
okf_version: "0.2"
type: Function
title: is_cancelled
description: Is this document cancelled?
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/is_cancelled
language: rust
---

# is_cancelled

Is this document cancelled?

## Signature

```rust
impl Document { pub fn is_cancelled(&self) -> bool }
```

## Visibility

- `pub`

## Docstring

Is this document cancelled?
[must_use]

## Source
Lines 68–70 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
