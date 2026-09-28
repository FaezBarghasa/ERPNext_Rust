---
okf_version: "0.2"
type: Class
title: ElectronicBatchRecord
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-manufacturing/src/ebr.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:49:30Z"
concept_id: crates/erp-manufacturing/src/ebr/ElectronicBatchRecord
language: rust
---

# ElectronicBatchRecord

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ElectronicBatchRecord
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `batch_number`
- `product_code`
- `manufacturing_date`
- `expiry_date`
- `steps`
- `is_released_for_distribution`

## Source
Lines 26–33 in `crates/erp-manufacturing/src/ebr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ebr](/crates/erp-manufacturing/src/ebr.md) |
