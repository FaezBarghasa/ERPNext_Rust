---
okf_version: "0.2"
type: Function
title: cosine_similarity
description: "Computes cosine similarity between two vector embeddings: $\\frac{A \\cdot B}{\\|A\\| \\|B\\|}$."
resource: crates/erp-cms/src/subtitles.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:55:58Z"
concept_id: crates/erp-cms/src/subtitles/cosine_similarity_1
language: rust
---

# cosine_similarity

Computes cosine similarity between two vector embeddings: $\frac{A \cdot B}{\|A\| \|B\|}$.

## Signature

```rust
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes cosine similarity between two vector embeddings: $\frac{A \cdot B}{\|A\| \|B\|}$.
[must_use]

## Source
Lines 32–52 in `crates/erp-cms/src/subtitles.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [subtitles](/crates/erp-cms/src/subtitles.md) |
