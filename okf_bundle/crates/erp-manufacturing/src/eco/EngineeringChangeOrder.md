---
okf_version: "0.2"
type: Class
title: EngineeringChangeOrder
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-manufacturing/src/eco.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:16:42Z"
concept_id: crates/erp-manufacturing/src/eco/EngineeringChangeOrder
language: rust
---

# EngineeringChangeOrder

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct EngineeringChangeOrder
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `eco_number`
- `item_code`
- `current_revision`
- `target_revision`
- `reason_for_change`
- `disposition`
- `status`
- `effective_date`
- `approvals`

## Source
Lines 23–33 in `crates/erp-manufacturing/src/eco.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [eco](/crates/erp-manufacturing/src/eco.md) |
