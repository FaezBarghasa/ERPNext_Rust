---
okf_version: "0.2"
type: Class
title: BomOperation
description: Operational routing step on a workstation.
resource: crates/erp-manufacturing/src/bom.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/bom/BomOperation
language: rust
---

# BomOperation

Operational routing step on a workstation.

## Signature

```rust
pub struct BomOperation
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Operational routing step on a workstation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `operation`
- `workstation`
- `time_in_mins`
- `hour_rate`

## Source
Lines 41–50 in `crates/erp-manufacturing/src/bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bom](/crates/erp-manufacturing/src/bom.md) |
