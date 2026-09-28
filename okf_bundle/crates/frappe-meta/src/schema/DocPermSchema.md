---
okf_version: "0.2"
type: Class
title: DocPermSchema
description: Role permission specification for DocType access control.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:40:46Z"
concept_id: crates/frappe-meta/src/schema/DocPermSchema
language: rust
---

# DocPermSchema

Role permission specification for DocType access control.

## Signature

```rust
pub struct DocPermSchema
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)`

## Visibility

- `pub`

## Docstring

Role permission specification for DocType access control.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]

## Methods

- `role`
- `read`
- `write`
- `create`
- `delete`
- `submit`
- `cancel`
- `amend`
- `report`
- `export`
- `permlevel`

## Source
Lines 115–148 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
