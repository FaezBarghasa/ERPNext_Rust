---
okf_version: "0.2"
type: Class
title: Batch
description: Batch tracking record with expiration management.
resource: crates/erp-inventory/src/batches.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:29Z"
concept_id: crates/erp-inventory/src/batches/Batch
language: rust
---

# Batch

Batch tracking record with expiration management.

## Signature

```rust
pub struct Batch
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Batch tracking record with expiration management.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `batch_id`
- `item_code`
- `mfg_date`
- `expiry_date`
- `remaining_qty`

## Source
Lines 51–62 in `crates/erp-inventory/src/batches.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [batches](/crates/erp-inventory/src/batches.md) |
