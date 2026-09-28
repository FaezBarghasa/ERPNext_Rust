---
okf_version: "0.2"
type: Function
title: prune_options
description: Prunes unavailable options dynamically given current user selections.
resource: crates/erp-crm/src/cpq.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/cpq/prune_options_1
language: rust
---

# prune_options

Prunes unavailable options dynamically given current user selections.

## Signature

```rust
pub fn prune_options(&self, selected: &[String]) -> HashSet<String>
```

## Visibility

- `pub`

## Docstring

Prunes unavailable options dynamically given current user selections.

## Source
Lines 46–58 in `crates/erp-crm/src/cpq.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [cpq](/crates/erp-crm/src/cpq.md) |
