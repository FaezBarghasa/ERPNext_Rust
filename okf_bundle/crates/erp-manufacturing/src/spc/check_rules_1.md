---
okf_version: "0.2"
type: Function
title: check_rules
description: "Checks the latest subgroups for Nelson & Western Electric rule violations."
resource: crates/erp-manufacturing/src/spc.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-manufacturing/src/spc/check_rules_1
language: rust
---

# check_rules

Checks the latest subgroups for Nelson & Western Electric rule violations.

## Signature

```rust
pub fn check_rules(subgroups: &[SpcSubgroup], limits: &ControlLimits) -> Vec<SpcRuleViolation>
```

## Visibility

- `pub`

## Docstring

Checks the latest subgroups for Nelson & Western Electric rule violations.

## Source
Lines 97–132 in `crates/erp-manufacturing/src/spc.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [spc](/crates/erp-manufacturing/src/spc.md) |
