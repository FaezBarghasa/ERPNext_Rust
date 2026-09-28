---
okf_version: "0.2"
type: Function
title: diff_boms
description: "Compares EBOM, MBOM, and SBOM to identify unmapped engineering deviations."
resource: crates/erp-manufacturing/src/quad_bom.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/quad_bom/diff_boms
language: rust
---

# diff_boms

Compares EBOM, MBOM, and SBOM to identify unmapped engineering deviations.

## Signature

```rust
impl QuadBomSynchronizer { pub fn diff_boms(
        ebom: &StructuredBom,
        mbom: &StructuredBom,
        sbom: Option<&StructuredBom>,
    ) -> Vec<BomDivergence> }
```

## Visibility

- `pub`

## Docstring

Compares EBOM, MBOM, and SBOM to identify unmapped engineering deviations.

## Source
Lines 46–96 in `crates/erp-manufacturing/src/quad_bom.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [quad_bom](/crates/erp-manufacturing/src/quad_bom.md) |
