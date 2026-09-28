---
okf_version: "0.2"
type: Function
title: record_signed_step
description: "[allow(clippy::too_many_arguments)]"
resource: crates/erp-manufacturing/src/ebr.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:49:30Z"
concept_id: crates/erp-manufacturing/src/ebr/record_signed_step_1
language: rust
---

# record_signed_step

[allow(clippy::too_many_arguments)]

## Signature

```rust
pub fn record_signed_step(
        &mut self,
        step_no: u32,
        op_name: String,
        params: serde_json::Value,
        operator_id: &str,
        operator_name: &str,
        operator_secret: &str,
        second_witness: Option<(&str, &str, &str)>, // (id, name, secret)
    )
```

## Decorators

- `allow(clippy::too_many_arguments)`

## Visibility

- `pub`

## Docstring

[allow(clippy::too_many_arguments)]

## Source
Lines 54–84 in `crates/erp-manufacturing/src/ebr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ebr](/crates/erp-manufacturing/src/ebr.md) |
