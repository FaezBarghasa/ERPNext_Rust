---
okf_version: "0.2"
type: Class
title: Lesson
description: Educational lesson content node.
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/Lesson
language: rust
---

# Lesson

Educational lesson content node.

## Signature

```rust
pub struct Lesson
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Educational lesson content node.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `title`
- `content_markdown`
- `video_url`

## Source
Lines 22–27 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
