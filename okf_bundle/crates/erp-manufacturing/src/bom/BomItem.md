---
okf_version: "0.2"
type: Class
title: BomItem
description: Raw material or sub-assembly line in a Bill of Materials.
resource: crates/erp-manufacturing/src/bom.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:07Z"
concept_id: crates/erp-manufacturing/src/bom/BomItem
language: rust
---

# BomItem

Raw material or sub-assembly line in a Bill of Materials.

## Signature

```rust
pub struct BomItem
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Raw material or sub-assembly line in a Bill of Materials.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `item_code`
- `qty`
- `scrap_percentage`
- `bom_no`

## Source
Lines 28–37 in `crates/erp-manufacturing/src/bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bom](/crates/erp-manufacturing/src/bom.md) |
