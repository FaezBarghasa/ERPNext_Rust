---
okf_version: "0.2"
type: Class
title: WbsNode
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-ppm/src/wbs.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:02:18Z"
concept_id: crates/erp-ppm/src/wbs/WbsNode
language: rust
---

# WbsNode

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct WbsNode
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `code`
- `name`
- `parent_id`
- `obs_department_id`
- `cbs_cost_code`
- `rbs_resource_pool_id`
- `planned_cost`
- `actual_cost`

## Source
Lines 7–17 in `crates/erp-ppm/src/wbs.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [wbs](/crates/erp-ppm/src/wbs.md) |
