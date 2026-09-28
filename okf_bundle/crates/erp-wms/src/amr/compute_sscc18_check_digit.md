---
okf_version: "0.2"
type: Function
title: compute_sscc18_check_digit
description: Generates standard GS1 SSCC-18 check digit.
resource: crates/erp-wms/src/amr.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-wms"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:43:54Z"
concept_id: crates/erp-wms/src/amr/compute_sscc18_check_digit
language: rust
---

# compute_sscc18_check_digit

Generates standard GS1 SSCC-18 check digit.

## Signature

```rust
impl HandlingUnit { pub fn compute_sscc18_check_digit(base17: &str) -> Option<u8> }
```

## Visibility

- `pub`

## Docstring

Generates standard GS1 SSCC-18 check digit.
[must_use]

## Source
Lines 48–60 in `crates/erp-wms/src/amr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [amr](/crates/erp-wms/src/amr.md) |
