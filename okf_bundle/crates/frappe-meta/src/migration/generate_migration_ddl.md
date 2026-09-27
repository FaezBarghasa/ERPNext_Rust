---
okf_version: "0.2"
type: Function
title: generate_migration_ddl
description: Generates transactional forward migration DDL and corresponding reverse rollback DDL.
resource: crates/frappe-meta/src/migration.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:46:12Z"
concept_id: crates/frappe-meta/src/migration/generate_migration_ddl
language: rust
---

# generate_migration_ddl

Generates transactional forward migration DDL and corresponding reverse rollback DDL.

## Signature

```rust
pub fn generate_migration_ddl(
    target: &DocTypeSchema,
    delta: &MigrationDelta,
) -> (Vec<String>, Vec<String>)
```

## Visibility

- `pub`

## Docstring

Generates transactional forward migration DDL and corresponding reverse rollback DDL.

## Source
Lines 94–154 in `crates/frappe-meta/src/migration.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [migration](/crates/frappe-meta/src/migration.md) |
| called_by | [test_schema_migration_diff_and_rollback](/crates/frappe-meta/tests/schema_compiler_tests/test_schema_migration_diff_and_rollback.md) |
