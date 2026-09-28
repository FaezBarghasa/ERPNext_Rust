---
okf_version: "0.2"
type: Function
title: compute
description: Executes both forward and backward CPM passes to compute floats and critical path.
resource: crates/erp-ppm/src/scheduler.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/scheduler/compute_1
language: rust
---

# compute

Executes both forward and backward CPM passes to compute floats and critical path.

## Signature

```rust
pub fn compute(&mut self) -> Result<i64, String>
```

## Visibility

- `pub`

## Docstring

Executes both forward and backward CPM passes to compute floats and critical path.

## Source
Lines 80–173 in `crates/erp-ppm/src/scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scheduler](/crates/erp-ppm/src/scheduler.md) |
