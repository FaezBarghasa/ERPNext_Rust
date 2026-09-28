---
okf_version: "0.2"
type: Function
title: transition_to
description: Mutates lead state.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:56:26Z"
concept_id: crates/erp-crm/src/pipeline/transition_to
language: rust
---

# transition_to

Mutates lead state.

## Signature

```rust
impl Lead { pub fn transition_to(&mut self, next_status: LeadStatus) -> Result<(), CrmError> }
```

## Visibility

- `pub`

## Docstring

Mutates lead state.

## Source
Lines 60–70 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
