---
okf_version: "0.2"
type: Function
title: complete_lesson
description: Marks a lesson as completed by the student.
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/complete_lesson
language: rust
---

# complete_lesson

Marks a lesson as completed by the student.

## Signature

```rust
impl StudentProgressTracker { pub fn complete_lesson(&mut self, lesson_id: &str) }
```

## Visibility

- `pub`

## Docstring

Marks a lesson as completed by the student.

## Source
Lines 79–81 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
