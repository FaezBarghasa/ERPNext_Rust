---
okf_version: "0.2"
type: Function
title: recognize_progress
description: Progressively recognizes revenue based on actual milestone completion percentage.
resource: crates/erp-crm/src/revops.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/revops/recognize_progress_1
language: rust
---

# recognize_progress

Progressively recognizes revenue based on actual milestone completion percentage.

## Signature

```rust
pub fn recognize_progress(
        pob: &mut PerformanceObligation,
        completion_percent: Decimal,
    ) -> Decimal
```

## Visibility

- `pub`

## Docstring

Progressively recognizes revenue based on actual milestone completion percentage.

## Source
Lines 56–67 in `crates/erp-crm/src/revops.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [revops](/crates/erp-crm/src/revops.md) |
