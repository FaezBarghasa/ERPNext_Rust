---
okf_version: "0.2"
type: Function
title: get_clause
description: "[must_use]"
resource: crates/erp-crm/src/clm.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/clm/get_clause
language: rust
---

# get_clause

[must_use]

## Signature

```rust
impl ClauseLibrary { pub fn get_clause(&self, title: &str, variant: ClauseVariant) -> Option<&ContractClause> }
```

## Visibility

- `pub`

## Docstring

[must_use]

## Source
Lines 66–71 in `crates/erp-crm/src/clm.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [clm](/crates/erp-crm/src/clm.md) |
