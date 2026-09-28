---
okf_version: "0.2"
type: Class
title: DeduplicatedStorage
description: Content-Addressable Storage (CAS) with SHA-256 deduplication and bounded-memory streaming.
resource: crates/frappe-storage/src/drive.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/drive/DeduplicatedStorage
language: rust
---

# DeduplicatedStorage

Content-Addressable Storage (CAS) with SHA-256 deduplication and bounded-memory streaming.

## Signature

```rust
pub struct DeduplicatedStorage
```

## Decorators

- `derive(Default, Clone)`

## Visibility

- `pub`

## Docstring

Content-Addressable Storage (CAS) with SHA-256 deduplication and bounded-memory streaming.
[derive(Default, Clone)]

## Methods

- `payloads`
- `files`
- `folders`

## Source
Lines 60–67 in `crates/frappe-storage/src/drive.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [drive](/crates/frappe-storage/src/drive.md) |
