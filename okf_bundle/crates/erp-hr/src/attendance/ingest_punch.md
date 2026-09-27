---
okf_version: "0.2"
type: Function
title: ingest_punch
description: Ingests an incoming punch event without blocking server request threads.
resource: crates/erp-hr/src/attendance.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:30:45Z"
concept_id: crates/erp-hr/src/attendance/ingest_punch
language: rust
---

# ingest_punch

Ingests an incoming punch event without blocking server request threads.

## Signature

```rust
impl BiometricIngestionGateway { pub fn ingest_punch(&self, punch: BiometricPunch) -> Result<(), HrError> }
```

## Visibility

- `pub`

## Docstring

Ingests an incoming punch event without blocking server request threads.

## Source
Lines 57–61 in `crates/erp-hr/src/attendance.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [attendance](/crates/erp-hr/src/attendance.md) |
