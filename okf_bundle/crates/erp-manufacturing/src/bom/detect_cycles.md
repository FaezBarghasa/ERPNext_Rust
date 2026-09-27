---
okf_version: "0.2"
type: Function
title: detect_cycles
description: Detects circular dependencies across BOM hierarchies using DFS cycle detection.
resource: crates/erp-manufacturing/src/bom.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:07Z"
concept_id: crates/erp-manufacturing/src/bom/detect_cycles
language: rust
---

# detect_cycles

Detects circular dependencies across BOM hierarchies using DFS cycle detection.

## Signature

```rust
impl BomEngine { pub fn detect_cycles(
        boms_by_item: &HashMap<String, Bom>,
    ) -> Result<(), ManufacturingError> }
```

## Visibility

- `pub`

## Docstring

Detects circular dependencies across BOM hierarchies using DFS cycle detection.

## Source
Lines 80–97 in `crates/erp-manufacturing/src/bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bom](/crates/erp-manufacturing/src/bom.md) |
