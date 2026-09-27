---
okf_version: "0.2"
type: Function
title: calculate_progress_percentage
description: "Computes percentage progress: $\\frac{\\text{Completed}}{\\text{Total}} \\times 100$."
resource: crates/erp-learning/src/lms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-learning"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:51:14Z"
concept_id: crates/erp-learning/src/lms/calculate_progress_percentage_1
language: rust
---

# calculate_progress_percentage

Computes percentage progress: $\frac{\text{Completed}}{\text{Total}} \times 100$.

## Signature

```rust
pub fn calculate_progress_percentage(&self, course: &Course) -> u32
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes percentage progress: $\frac{\text{Completed}}{\text{Total}} \times 100$.
[must_use]

## Source
Lines 85–101 in `crates/erp-learning/src/lms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lms](/crates/erp-learning/src/lms.md) |
