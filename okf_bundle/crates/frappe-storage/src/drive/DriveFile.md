---
okf_version: "0.2"
type: Class
title: DriveFile
description: Metadata record for virtual drive files.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T18:51:07Z"
concept_id: crates/frappe-storage/src/drive/DriveFile
language: rust
---

# DriveFile

Metadata record for virtual drive files.

## Signature

```rust
pub struct DriveFile
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Metadata record for virtual drive files.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `file_name`
- `folder_id`
- `content_hash`
- `file_size`
- `mime_type`
- `owner`

## Source
Lines 41–56 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
