---
okf_version: "0.2"
type: Function
title: compile_field_ddl
resource: crates/frappe-meta/src/schema_compiler.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:38Z"
concept_id: crates/frappe-meta/src/schema_compiler/compile_field_ddl
language: rust
---

# compile_field_ddl

## Signature

```rust
fn compile_field_ddl(table: &str, field: &DocFieldSchema) -> Result<String, SchemaError>
```

## Source
Lines 49–90 in `crates/frappe-meta/src/schema_compiler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema_compiler](/crates/frappe-meta/src/schema_compiler.md) |
| called_by | [compile_to_surrealql](/crates/frappe-meta/src/schema_compiler/compile_to_surrealql.md) |
