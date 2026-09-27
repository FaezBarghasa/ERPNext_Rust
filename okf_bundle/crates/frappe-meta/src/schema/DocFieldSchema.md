---
okf_version: "0.2"
type: Class
title: DocFieldSchema
description: Structural schema definition for an individual DocField.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:15Z"
concept_id: crates/frappe-meta/src/schema/DocFieldSchema
language: rust
---

# DocFieldSchema

Structural schema definition for an individual DocField.

## Signature

```rust
pub struct DocFieldSchema
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq)`

## Visibility

- `pub`

## Docstring

Structural schema definition for an individual DocField.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

## Methods

- `fieldname`
- `fieldtype`
- `label`
- `reqd`
- `unique`
- `read_only`
- `hidden`
- `in_list_view`
- `options`
- `default_value`

## Source
Lines 152–180 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
