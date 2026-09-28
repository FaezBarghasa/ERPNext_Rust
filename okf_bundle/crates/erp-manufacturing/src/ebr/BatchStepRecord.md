---
okf_version: "0.2"
type: Class
title: BatchStepRecord
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
concept_id: crates/erp-manufacturing/src/ebr/BatchStepRecord
language: rust
---

# BatchStepRecord

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct BatchStepRecord
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `step_number`
- `operation_name`
- `parameter_values`
- `primary_operator_signature`
- `second_witness_signature`
- `is_verified`

## Source
Lines 16–23 in `crates/erp-manufacturing/src/ebr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ebr](/crates/erp-manufacturing/src/ebr.md) |
