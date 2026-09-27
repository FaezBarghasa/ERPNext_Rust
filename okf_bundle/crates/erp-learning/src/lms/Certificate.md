---
okf_version: "0.2"
type: Class
title: Certificate
description: Verified completion certificate (Milestone 4.7).
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/Certificate
language: rust
---

# Certificate

Verified completion certificate (Milestone 4.7).

## Signature

```rust
pub struct Certificate
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Verified completion certificate (Milestone 4.7).
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `certificate_no`
- `student_id`
- `course_id`
- `issue_date`
- `final_grade_score`

## Source
Lines 57–63 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
