---
okf_version: "0.2"
type: Function
title: counter_stmt
description: SurrealQL atomic counter statement for concurrent inserts.
resource: crates/frappe-meta/src/naming.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-meta"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T14:13:33Z"
concept_id: crates/frappe-meta/src/naming/counter_stmt_1
language: rust
---

# counter_stmt

SurrealQL atomic counter statement for concurrent inserts.

## Signature

```rust
pub fn counter_stmt(table: &str) -> String
```

## Visibility

- `pub`

## Docstring

SurrealQL atomic counter statement for concurrent inserts.

## Source
Lines 19–21 in `crates/frappe-meta/src/naming.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [naming](/crates/frappe-meta/src/naming.md) |
