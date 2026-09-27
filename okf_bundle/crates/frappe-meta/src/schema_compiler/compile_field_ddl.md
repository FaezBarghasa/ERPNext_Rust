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
