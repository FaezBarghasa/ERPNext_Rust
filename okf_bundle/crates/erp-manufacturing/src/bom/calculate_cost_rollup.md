---
okf_version: "0.2"
type: Function
title: calculate_cost_rollup
description: Recursive Multi-Level BOM Cost Rollup Engine (Milestone 3.2).
resource: crates/erp-manufacturing/src/bom.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:07Z"
concept_id: crates/erp-manufacturing/src/bom/calculate_cost_rollup
language: rust
---

# calculate_cost_rollup

Recursive Multi-Level BOM Cost Rollup Engine (Milestone 3.2).

## Signature

```rust
impl BomEngine { pub fn calculate_cost_rollup(
        item_code: &str,
        boms_by_item: &HashMap<String, Bom>,
        item_valuation_rates: &HashMap<String, Decimal>,
    ) -> Result<Decimal, ManufacturingError> }
```

## Visibility

- `pub`

## Docstring

Recursive Multi-Level BOM Cost Rollup Engine (Milestone 3.2).
Sums raw material unit costs + workstation operating runtimes across all nesting depths.

## Source
Lines 132–173 in `crates/erp-manufacturing/src/bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bom](/crates/erp-manufacturing/src/bom.md) |
