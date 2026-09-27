---
okf_version: "0.2"
type: Class
title: FieldType
description: Primitives for DocType field types.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:15Z"
concept_id: crates/frappe-meta/src/schema/FieldType
language: rust
---

# FieldType

Primitives for DocType field types.

## Signature

```rust
pub enum FieldType
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq)`
- `serde(tag = "type", content = "options")`

## Visibility

- `pub`

## Docstring

Primitives for DocType field types.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
[serde(tag = "type", content = "options")]

## Methods

- `target_doctype`
- `link_field`
- `child_doctype`
- `options`

## Source
Lines 31–76 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
