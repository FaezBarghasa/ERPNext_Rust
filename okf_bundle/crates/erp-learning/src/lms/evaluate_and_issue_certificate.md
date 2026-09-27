---
okf_version: "0.2"
type: Function
title: evaluate_and_issue_certificate
description: "Evaluates assessment and auto-issues certificate if 100% complete and passing grade achieved."
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/evaluate_and_issue_certificate
language: rust
---

# evaluate_and_issue_certificate

Evaluates assessment and auto-issues certificate if 100% complete and passing grade achieved.

## Signature

```rust
impl StudentProgressTracker { pub fn evaluate_and_issue_certificate(
        &self,
        student_id: &str,
        course: &Course,
        assessment_score: u32,
        issue_date: NaiveDate,
    ) -> Result<Certificate, LmsError> }
```

## Visibility

- `pub`

## Docstring

Evaluates assessment and auto-issues certificate if 100% complete and passing grade achieved.

## Source
Lines 104–133 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
