---
okf_version: "0.2"
type: Function
title: compensate
resource: crates/frappe-framework/src/saga.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/saga/compensate_1
language: rust
---

# compensate

## Signature

```rust
fn compensate(&self, _payload: &serde_json::Value) -> Result<(), String>
```

## Source
Lines 109–112 in `crates/frappe-framework/src/saga.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [saga](/crates/frappe-framework/src/saga.md) |
