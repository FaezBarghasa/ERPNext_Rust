---
okf_version: "0.2"
type: Function
title: calculate_lead_score
description: Computes weighted lead priority score (0..100) based on deal size and engagement.
resource: crates/erp-crm/src/scoring.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:36:06Z"
concept_id: crates/erp-crm/src/scoring/calculate_lead_score
language: rust
---

# calculate_lead_score

Computes weighted lead priority score (0..100) based on deal size and engagement.

## Signature

```rust
pub fn calculate_lead_score(
    engagement_count: u32,
    estimated_deal_value: Decimal,
    source_weight: Decimal,
) -> Decimal
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes weighted lead priority score (0..100) based on deal size and engagement.
[must_use]

## Source
Lines 5–16 in `crates/erp-crm/src/scoring.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scoring](/crates/erp-crm/src/scoring.md) |
| called_by | [test_lead_score_bounds](/crates/erp-crm/src/scoring/test_lead_score_bounds.md) |
