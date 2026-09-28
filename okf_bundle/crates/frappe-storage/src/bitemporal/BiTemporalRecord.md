---
okf_version: "0.2"
type: Class
title: BiTemporalRecord
description: A bi-temporal EAV record storing both system commit time and business fact valid time.
resource: crates/frappe-storage/src/bitemporal.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/bitemporal/BiTemporalRecord
language: rust
---

# BiTemporalRecord

A bi-temporal EAV record storing both system commit time and business fact valid time.

## Signature

```rust
pub struct BiTemporalRecord
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

A bi-temporal EAV record storing both system commit time and business fact valid time.
[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `doc_id`
- `doctype`
- `system_time`
- `valid_time`
- `attributes`
- `prev_hash`
- `merkle_hash`

## Source
Lines 43–52 in `crates/frappe-storage/src/bitemporal.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bitemporal](/crates/frappe-storage/src/bitemporal.md) |
