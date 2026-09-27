---
okf_version: "0.2"
type: Function
title: diff_schema
description: Calculates the structural diff between a target DocType and live database schema.
resource: crates/frappe-meta/src/migration.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:46:12Z"
concept_id: crates/frappe-meta/src/migration/diff_schema
language: rust
---

# diff_schema

Calculates the structural diff between a target DocType and live database schema.

## Signature

```rust
pub fn diff_schema(
    target: &DocTypeSchema,
    existing_db: &ExistingDatabaseSchema,
) -> Result<MigrationDelta, SchemaError>
```

## Visibility

- `pub`

## Docstring

Calculates the structural diff between a target DocType and live database schema.

## Source
Lines 47–91 in `crates/frappe-meta/src/migration.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [migration](/crates/frappe-meta/src/migration.md) |
| called_by | [test_schema_migration_diff_and_rollback](/crates/frappe-meta/tests/schema_compiler_tests/test_schema_migration_diff_and_rollback.md) |
