---
okf_version: "0.2"
type: Function
title: create_folder
description: Creates a virtual drive folder.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:28:39Z"
concept_id: crates/frappe-storage/src/drive/create_folder
language: rust
---

# create_folder

Creates a virtual drive folder.

## Signature

```rust
impl DeduplicatedStorage { pub fn create_folder(&self, folder: DriveFolder) -> Result<(), StorageError> }
```

## Visibility

- `pub`

## Docstring

Creates a virtual drive folder.

## Source
Lines 142–149 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
