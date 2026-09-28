---
okf_version: "0.2"
type: Class
title: CorporateHierarchy
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-crm/src/buying_center.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/buying_center/CorporateHierarchy
language: rust
---

# CorporateHierarchy

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct CorporateHierarchy
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `entity_id`
- `parent_entity_id`
- `global_ultimate_owner_id`
- `domestic_ultimate_owner_id`
- `credit_limit`
- `consolidated_pricing_tier`

## Source
Lines 36–43 in `crates/erp-crm/src/buying_center.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [buying_center](/crates/erp-crm/src/buying_center.md) |
