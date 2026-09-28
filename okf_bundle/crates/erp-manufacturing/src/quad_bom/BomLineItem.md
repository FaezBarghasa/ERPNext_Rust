---
okf_version: "0.2"
type: Class
title: BomLineItem
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-manufacturing/src/quad_bom.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/quad_bom/BomLineItem
language: rust
---

# BomLineItem

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct BomLineItem
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `item_code`
- `description`
- `quantity`
- `uom`
- `is_field_replaceable`
- `is_phantom_subassembly`

## Source
Lines 15–22 in `crates/erp-manufacturing/src/quad_bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [quad_bom](/crates/erp-manufacturing/src/quad_bom.md) |
