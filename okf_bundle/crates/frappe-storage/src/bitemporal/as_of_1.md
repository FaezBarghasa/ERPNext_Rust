---
okf_version: "0.2"
type: Function
title: as_of
description: Builds a SurrealQL statement fetching records as of a specific system time and valid time.
resource: crates/frappe-storage/src/bitemporal.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/bitemporal/as_of_1
language: rust
---

# as_of

Builds a SurrealQL statement fetching records as of a specific system time and valid time.

## Signature

```rust
pub fn as_of(
        doctype: &str,
        doc_id: &str,
        system_as_of: DateTime<Utc>,
        valid_as_of: DateTime<Utc>,
    ) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Builds a SurrealQL statement fetching records as of a specific system time and valid time.
[must_use]

## Source
Lines 104–118 in `crates/frappe-storage/src/bitemporal.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bitemporal](/crates/frappe-storage/src/bitemporal.md) |
