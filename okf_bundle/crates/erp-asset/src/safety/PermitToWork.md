---
okf_version: "0.2"
type: Class
title: PermitToWork
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-asset/src/safety.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-asset"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-asset/src/safety/PermitToWork
language: rust
---

# PermitToWork

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct PermitToWork
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `permit_id`
- `asset_id`
- `work_order_id`
- `work_type`
- `loto_tags`
- `gas_test_passed`
- `safety_marshal_signature`
- `is_active`

## Source
Lines 26–35 in `crates/erp-asset/src/safety.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [safety](/crates/erp-asset/src/safety.md) |
