---
okf_version: "0.2"
type: Function
title: from_schema
description: Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.
resource: crates/desk-components/src/forms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/desk-components/src/forms/from_schema_1
language: rust
---

# from_schema

Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.

## Signature

```rust
pub fn from_schema(schema: &DocTypeSchema) -> Self
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Compiles a server DocTypeSchema AST into a client-side DynamicFormModel.
[must_use]

## Source
Lines 59–134 in `crates/desk-components/src/forms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [forms](/crates/desk-components/src/forms.md) |
