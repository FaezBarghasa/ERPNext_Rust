---
okf_version: "0.2"
type: Function
title: physical_payload_count
description: Returns the number of distinct physical binary payloads stored in memory.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:28:39Z"
concept_id: crates/frappe-storage/src/drive/physical_payload_count_1
language: rust
---

# physical_payload_count

Returns the number of distinct physical binary payloads stored in memory.

## Signature

```rust
pub fn physical_payload_count(&self) -> usize
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Returns the number of distinct physical binary payloads stored in memory.
[must_use]

## Source
Lines 158–160 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
