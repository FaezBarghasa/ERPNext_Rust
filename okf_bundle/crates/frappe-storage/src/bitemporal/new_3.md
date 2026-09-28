---
okf_version: "0.2"
type: Function
title: new
resource: crates/frappe-storage/src/bitemporal.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/bitemporal/new_3
language: rust
---

# new

## Signature

```rust
pub fn new(
        id: String,
        doc_id: String,
        doctype: String,
        system_time: TimeInterval,
        valid_time: TimeInterval,
        attributes: BTreeMap<String, serde_json::Value>,
        prev_hash: String,
    ) -> Self
```

## Visibility

- `pub`

## Source
Lines 55–76 in `crates/frappe-storage/src/bitemporal.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bitemporal](/crates/frappe-storage/src/bitemporal.md) |
