---
okf_version: "0.2"
type: Function
title: run_wat
description: "Compile + run a wat module exporting `run() -> i32`; proves fuel termination."
resource: crates/frappe-framework/src/wasmtime_sandbox.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:23:45Z"
concept_id: crates/frappe-framework/src/wasmtime_sandbox/run_wat
language: rust
---

# run_wat

Compile + run a wat module exporting `run() -> i32`; proves fuel termination.

## Signature

```rust
impl RealSandbox { pub fn run_wat(&self, wat: &str) -> Result<i32> }
```

## Visibility

- `pub`

## Docstring

Compile + run a wat module exporting `run() -> i32`; proves fuel termination.

## Source
Lines 48–62 in `crates/frappe-framework/src/wasmtime_sandbox.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [wasmtime_sandbox](/crates/frappe-framework/src/wasmtime_sandbox.md) |
