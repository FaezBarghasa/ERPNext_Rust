---
okf_version: "0.2"
type: Function
title: table_growing
resource: crates/frappe-framework/src/wasmtime_sandbox.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:23:45Z"
concept_id: crates/frappe-framework/src/wasmtime_sandbox/table_growing
language: rust
---

# table_growing

## Signature

```rust
impl Limits { fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> }
```

## Source
Lines 26–33 in `crates/frappe-framework/src/wasmtime_sandbox.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [wasmtime_sandbox](/crates/frappe-framework/src/wasmtime_sandbox.md) |
