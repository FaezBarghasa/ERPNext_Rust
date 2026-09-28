---
okf_version: "0.2"
type: Function
title: execute_saga
description: "Executes a list of saga steps sequentially. On failure, triggers backward compensation."
resource: crates/frappe-framework/src/saga.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/saga/execute_saga
language: rust
---

# execute_saga

Executes a list of saga steps sequentially. On failure, triggers backward compensation.

## Signature

```rust
impl SagaCoordinator { pub fn execute_saga(
        &mut self,
        saga_id: &str,
        transactions: &[SagaTransaction],
    ) -> Result<Vec<serde_json::Value>, String> }
```

## Visibility

- `pub`

## Docstring

Executes a list of saga steps sequentially. On failure, triggers backward compensation.

## Source
Lines 53–86 in `crates/frappe-framework/src/saga.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [saga](/crates/frappe-framework/src/saga.md) |
