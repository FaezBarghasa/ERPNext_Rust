---
okf_version: "0.2"
type: Function
title: get_folder
description: Retrieves a virtual drive folder by ID.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:28:39Z"
concept_id: crates/frappe-storage/src/drive/get_folder
language: rust
---

# get_folder

Retrieves a virtual drive folder by ID.

## Signature

```rust
impl DeduplicatedStorage { pub fn get_folder(&self, folder_id: &str) -> Option<DriveFolder> }
```

## Visibility

- `pub`

## Docstring

Retrieves a virtual drive folder by ID.

## Source
Lines 152–154 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
