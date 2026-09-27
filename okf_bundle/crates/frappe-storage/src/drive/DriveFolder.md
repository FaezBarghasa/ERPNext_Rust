---
okf_version: "0.2"
type: Class
title: DriveFolder
description: Metadata record for virtual drive folders.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:28:39Z"
concept_id: crates/frappe-storage/src/drive/DriveFolder
language: rust
---

# DriveFolder

Metadata record for virtual drive folders.

## Signature

```rust
pub struct DriveFolder
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Metadata record for virtual drive folders.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `name`
- `parent_id`
- `owner`
- `is_public`

## Source
Lines 26–37 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
