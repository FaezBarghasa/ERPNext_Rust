---
okf_version: "0.2"
type: Function
title: match_rule
resource: crates/frappe-framework/src/dmn.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/dmn/match_rule
language: rust
---

# match_rule

## Signature

```rust
impl DecisionTable { fn match_rule(&self, rule: &DecisionRule, inputs: &[serde_json::Value]) -> bool }
```

## Source
Lines 94–171 in `crates/frappe-framework/src/dmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [dmn](/crates/frappe-framework/src/dmn.md) |
