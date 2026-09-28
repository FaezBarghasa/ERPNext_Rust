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
timestamp: "2026-09-27T21:40:27Z"
concept_id: crates/frappe-meta/src/naming/counter_stmt
language: rust
---

# counter_stmt

SurrealQL atomic counter statement for concurrent inserts.

## Signature

```rust
impl NamingSeriesParser { pub fn counter_stmt(table: &str) -> String }
```

## Visibility

- `pub`

## Docstring

SurrealQL atomic counter statement for concurrent inserts.

## Source
Lines 23–28 in `crates/frappe-meta/src/naming.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [naming](/crates/frappe-meta/src/naming.md) |
