---
okf_version: "0.2"
type: Class
title: ChangeOrder
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-ppm/src/construction.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/construction/ChangeOrder
language: rust
---

# ChangeOrder

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ChangeOrder
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `co_type`
- `description`
- `cost_impact`
- `schedule_impact_days`
- `is_approved`

## Source
Lines 54–61 in `crates/erp-ppm/src/construction.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [construction](/crates/erp-ppm/src/construction.md) |
