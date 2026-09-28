---
okf_version: "0.2"
type: Function
title: execute
resource: crates/frappe-framework/src/saga.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/saga/execute_1
language: rust
---

# execute

## Signature

```rust
fn execute(&self, _payload: &serde_json::Value) -> Result<serde_json::Value, String>
```

## Source
Lines 101–107 in `crates/frappe-framework/src/saga.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [saga](/crates/frappe-framework/src/saga.md) |
