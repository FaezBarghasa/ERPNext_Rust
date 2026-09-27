---
okf_version: "0.2"
type: Function
title: validate
description: Validates the DocType schema AST according to strict architectural invariants.
resource: crates/frappe-meta/src/schema.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:45:15Z"
concept_id: crates/frappe-meta/src/schema/validate_1
language: rust
---

# validate

Validates the DocType schema AST according to strict architectural invariants.

## Signature

```rust
pub fn validate(&self) -> Result<(), SchemaError>
```

## Visibility

- `pub`

## Docstring

Validates the DocType schema AST according to strict architectural invariants.

## Source
Lines 216–254 in `crates/frappe-meta/src/schema.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [schema](/crates/frappe-meta/src/schema.md) |
