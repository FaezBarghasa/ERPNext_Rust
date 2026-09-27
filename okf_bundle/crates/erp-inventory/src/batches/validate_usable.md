---
okf_version: "0.2"
type: Function
title: validate_usable
description: Validates whether the batch is active and unexpired as of a given posting date.
resource: crates/erp-inventory/src/batches.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:29Z"
concept_id: crates/erp-inventory/src/batches/validate_usable
language: rust
---

# validate_usable

Validates whether the batch is active and unexpired as of a given posting date.

## Signature

```rust
impl Batch { pub fn validate_usable(&self, as_of: NaiveDate) -> Result<(), InventoryError> }
```

## Visibility

- `pub`

## Docstring

Validates whether the batch is active and unexpired as of a given posting date.

## Source
Lines 66–76 in `crates/erp-inventory/src/batches.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [batches](/crates/erp-inventory/src/batches.md) |
