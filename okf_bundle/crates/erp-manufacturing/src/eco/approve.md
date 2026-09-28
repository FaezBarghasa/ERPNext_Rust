---
okf_version: "0.2"
type: Function
title: approve
resource: crates/erp-manufacturing/src/eco.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:16:42Z"
concept_id: crates/erp-manufacturing/src/eco/approve
language: rust
---

# approve

## Signature

```rust
impl EngineeringChangeOrder { pub fn approve(&mut self, approver_id: &str) -> Result<(), String> }
```

## Visibility

- `pub`

## Source
Lines 59–68 in `crates/erp-manufacturing/src/eco.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [eco](/crates/erp-manufacturing/src/eco.md) |
