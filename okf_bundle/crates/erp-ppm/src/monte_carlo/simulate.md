---
okf_version: "0.2"
type: Function
title: simulate
description: Evaluates stochastic task durations over N iterations using Stratified Latin Hypercube Sampling.
resource: crates/erp-ppm/src/monte_carlo.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/monte_carlo/simulate
language: rust
---

# simulate

Evaluates stochastic task durations over N iterations using Stratified Latin Hypercube Sampling.

## Signature

```rust
impl MonteCarloSimulator { pub fn simulate(
        tasks: &[TaskRiskProfile],
        iterations: usize,
    ) -> Result<MonteCarloSummary, String> }
```

## Visibility

- `pub`

## Docstring

Evaluates stochastic task durations over N iterations using Stratified Latin Hypercube Sampling.

## Source
Lines 45–120 in `crates/erp-ppm/src/monte_carlo.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [monte_carlo](/crates/erp-ppm/src/monte_carlo.md) |
