---
okf_version: "0.2"
type: Function
title: compile_to_surrealql
description: "Compiles a strongly typed `DocTypeSchema` into a sequence of enforceable SurrealQL DDL statements."
resource: crates/frappe-meta/src/schema_compiler.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:38Z"
concept_id: crates/frappe-meta/src/schema_compiler/compile_to_surrealql
language: rust
---

# compile_to_surrealql

Compiles a strongly typed `DocTypeSchema` into a sequence of enforceable SurrealQL DDL statements.

## Signature

```rust
pub fn compile_to_surrealql(schema: &DocTypeSchema) -> Result<Vec<String>, SchemaError>
```

## Visibility

- `pub`

## Docstring

Compiles a strongly typed `DocTypeSchema` into a sequence of enforceable SurrealQL DDL statements.

## Source
Lines 4–47 in `crates/frappe-meta/src/schema_compiler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema_compiler](/crates/frappe-meta/src/schema_compiler.md) |
| calls | [compile_field_ddl](/crates/frappe-meta/src/schema_compiler/compile_field_ddl.md) |
| called_by | [test_schema_deserialization_and_compilation](/crates/frappe-meta/tests/schema_compiler_tests/test_schema_deserialization_and_compilation.md) |
