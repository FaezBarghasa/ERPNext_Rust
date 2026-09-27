---
okf_version: "0.2"
type: Class
title: MigrationDelta
description: Calculated structural difference between target schema and live database.
resource: crates/frappe-meta/src/migration.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:46:12Z"
concept_id: crates/frappe-meta/src/migration/MigrationDelta
language: rust
---

# MigrationDelta

Calculated structural difference between target schema and live database.

## Signature

```rust
pub struct MigrationDelta
```

## Decorators

- `derive(Debug, Clone, PartialEq, Default)`

## Visibility

- `pub`

## Docstring

Calculated structural difference between target schema and live database.
[derive(Debug, Clone, PartialEq, Default)]

## Methods

- `table_name`
- `is_new_table`
- `added_fields`
- `removed_fields`
- `modified_fields`

## Source
Lines 33–44 in `crates/frappe-meta/src/migration.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [migration](/crates/frappe-meta/src/migration.md) |
