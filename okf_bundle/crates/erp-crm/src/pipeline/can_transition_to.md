---
okf_version: "0.2"
type: Function
title: can_transition_to
description: Validates state transition progression.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:34:58Z"
concept_id: crates/erp-crm/src/pipeline/can_transition_to
language: rust
---

# can_transition_to

Validates state transition progression.

## Signature

```rust
impl LeadStatus { pub fn can_transition_to(self, target: LeadStatus) -> bool }
```

## Visibility

- `pub`

## Docstring

Validates state transition progression.

## Source
Lines 40–48 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
