---
okf_version: "0.2"
type: Function
title: store_file
description: "Stores a binary payload, deduplicating based on SHA-256 content hash."
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:07Z"
concept_id: crates/frappe-storage/src/drive/store_file
language: rust
---

# store_file

Stores a binary payload, deduplicating based on SHA-256 content hash.

## Signature

```rust
impl DeduplicatedStorage { pub fn store_file(
        &self,
        file_id: String,
        file_name: String,
        folder_id: Option<String>,
        mime_type: String,
        owner: String,
        data: &[u8],
    ) -> Result<(DriveFile, bool), StorageError> }
```

## Visibility

- `pub`

## Docstring

Stores a binary payload, deduplicating based on SHA-256 content hash.
Returns (DriveFile, is_duplicate).

## Source
Lines 78–118 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
