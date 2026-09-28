---
okf_version: "0.2"
type: Function
title: is_draft
description: Is this document a draft?
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/is_draft
language: rust
---

# is_draft

Is this document a draft?

## Signature

```rust
impl Document { pub fn is_draft(&self) -> bool }
```

## Visibility

- `pub`

## Docstring

Is this document a draft?
[must_use]

## Source
Lines 56–58 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
