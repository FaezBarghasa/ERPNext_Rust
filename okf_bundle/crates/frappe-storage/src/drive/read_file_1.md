---
okf_version: "0.2"
type: Function
title: read_file
description: Retrieves file bytes by file ID.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:07Z"
concept_id: crates/frappe-storage/src/drive/read_file_1
language: rust
---

# read_file

Retrieves file bytes by file ID.

## Signature

```rust
pub fn read_file(&self, file_id: &str) -> Result<Vec<u8>, StorageError>
```

## Visibility

- `pub`

## Docstring

Retrieves file bytes by file ID.

## Source
Lines 121–139 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
