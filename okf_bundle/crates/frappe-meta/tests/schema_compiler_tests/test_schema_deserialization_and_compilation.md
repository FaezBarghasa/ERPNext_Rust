---
okf_version: "0.2"
type: Function
title: test_schema_deserialization_and_compilation
description: "[test]"
resource: crates/frappe-meta/tests/schema_compiler_tests.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:48:45Z"
concept_id: crates/frappe-meta/tests/schema_compiler_tests/test_schema_deserialization_and_compilation
language: rust
---

# test_schema_deserialization_and_compilation

[test]

## Signature

```rust
fn test_schema_deserialization_and_compilation()
```

## Decorators

- `test`

## Docstring

[test]

## Source
Lines 84–93 in `crates/frappe-meta/tests/schema_compiler_tests.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema_compiler_tests](/crates/frappe-meta/tests/schema_compiler_tests.md) |
| calls | [create_customer_schema](/crates/frappe-meta/tests/schema_compiler_tests/create_customer_schema.md) |
| calls | [compile_to_surrealql](/crates/frappe-meta/src/schema_compiler/compile_to_surrealql.md) |
