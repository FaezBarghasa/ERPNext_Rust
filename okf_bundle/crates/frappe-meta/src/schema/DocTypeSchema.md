---
okf_version: "0.2"
type: Class
title: DocTypeSchema
description: Abstract Syntax Tree (AST) definition for an entire DocType schema.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:40:46Z"
concept_id: crates/frappe-meta/src/schema/DocTypeSchema
language: rust
---

# DocTypeSchema

Abstract Syntax Tree (AST) definition for an entire DocType schema.

## Signature

```rust
pub struct DocTypeSchema
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq)`

## Visibility

- `pub`

## Docstring

Abstract Syntax Tree (AST) definition for an entire DocType schema.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

## Methods

- `name`
- `module`
- `is_single`
- `is_submittable`
- `track_changes`
- `naming_rule`
- `fields`
- `permissions`

## Source
Lines 184–206 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
