---
okf_version: "0.2"
type: Function
title: total_lesson_count
description: Returns total lesson count across all modules.
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/total_lesson_count
language: rust
---

# total_lesson_count

Returns total lesson count across all modules.

## Signature

```rust
impl Course { pub fn total_lesson_count(&self) -> usize }
```

## Visibility

- `pub`

## Docstring

Returns total lesson count across all modules.
[must_use]

## Source
Lines 50–52 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
