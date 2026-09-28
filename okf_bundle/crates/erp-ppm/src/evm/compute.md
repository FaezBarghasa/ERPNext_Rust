---
okf_version: "0.2"
type: Function
title: compute
resource: crates/erp-ppm/src/evm.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/evm/compute
language: rust
---

# compute

## Signature

```rust
impl EvmEngine { pub fn compute(inputs: &EvmInputs) -> Result<EvmMetrics, String> }
```

## Visibility

- `pub`

## Source
Lines 30–75 in `crates/erp-ppm/src/evm.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [evm](/crates/erp-ppm/src/evm.md) |
