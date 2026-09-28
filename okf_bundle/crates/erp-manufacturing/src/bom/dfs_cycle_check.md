---
okf_version: "0.2"
type: Function
title: dfs_cycle_check
resource: crates/erp-manufacturing/src/bom.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/bom/dfs_cycle_check
language: rust
---

# dfs_cycle_check

## Signature

```rust
impl BomEngine { fn dfs_cycle_check(
        current_item: &str,
        boms_by_item: &HashMap<String, Bom>,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Result<(), ManufacturingError> }
```

## Source
Lines 92–121 in `crates/erp-manufacturing/src/bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bom](/crates/erp-manufacturing/src/bom.md) |
