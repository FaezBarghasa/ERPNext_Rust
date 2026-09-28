---
okf_version: "0.2"
type: Function
title: evaluate
description: Evaluates inputs against the decision table and applies hit policies.
resource: crates/frappe-framework/src/dmn.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/dmn/evaluate_1
language: rust
---

# evaluate

Evaluates inputs against the decision table and applies hit policies.

## Signature

```rust
pub fn evaluate(&self, inputs: &[serde_json::Value]) -> Result<Vec<serde_json::Value>, String>
```

## Visibility

- `pub`

## Docstring

Evaluates inputs against the decision table and applies hit policies.

## Source
Lines 62–92 in `crates/frappe-framework/src/dmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [dmn](/crates/frappe-framework/src/dmn.md) |
