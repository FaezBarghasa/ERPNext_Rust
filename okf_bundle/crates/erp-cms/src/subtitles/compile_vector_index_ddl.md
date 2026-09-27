---
okf_version: "0.2"
type: Function
title: compile_vector_index_ddl
description: Emits SurrealDB v3 HNSW Vector Index DDL statements.
resource: crates/erp-cms/src/subtitles.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-cms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:55:58Z"
concept_id: crates/erp-cms/src/subtitles/compile_vector_index_ddl
language: rust
---

# compile_vector_index_ddl

Emits SurrealDB v3 HNSW Vector Index DDL statements.

## Signature

```rust
impl SubtitleSearchEngine { pub fn compile_vector_index_ddl() -> Vec<String> }
```

## Visibility

- `pub`

## Docstring

Emits SurrealDB v3 HNSW Vector Index DDL statements.
[must_use]

## Source
Lines 18–28 in `crates/erp-cms/src/subtitles.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [subtitles](/crates/erp-cms/src/subtitles.md) |
