---
okf_version: "0.2"
type: Function
title: add_rule
resource: crates/frappe-framework/src/dmn.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/dmn/add_rule
language: rust
---

# add_rule

## Signature

```rust
impl DecisionTable { pub fn add_rule(&mut self, conditions: Vec<ConditionOp>, outputs: Vec<serde_json::Value>) }
```

## Visibility

- `pub`

## Source
Lines 54–59 in `crates/frappe-framework/src/dmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [dmn](/crates/frappe-framework/src/dmn.md) |
