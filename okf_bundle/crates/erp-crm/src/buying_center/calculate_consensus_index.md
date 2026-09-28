---
okf_version: "0.2"
type: Function
title: calculate_consensus_index
description: Evaluates account consensus score based on Champions vs Blockers weighted by influence.
resource: crates/erp-crm/src/buying_center.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/buying_center/calculate_consensus_index
language: rust
---

# calculate_consensus_index

Evaluates account consensus score based on Champions vs Blockers weighted by influence.

## Signature

```rust
impl BuyingCenterGraph { pub fn calculate_consensus_index(&self) -> f64 }
```

## Visibility

- `pub`

## Docstring

Evaluates account consensus score based on Champions vs Blockers weighted by influence.
[must_use]

## Source
Lines 69–83 in `crates/erp-crm/src/buying_center.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [buying_center](/crates/erp-crm/src/buying_center.md) |
