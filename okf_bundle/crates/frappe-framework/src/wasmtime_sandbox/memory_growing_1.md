---
okf_version: "0.2"
type: Function
title: memory_growing
resource: crates/frappe-framework/src/wasmtime_sandbox.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:23:45Z"
concept_id: crates/frappe-framework/src/wasmtime_sandbox/memory_growing_1
language: rust
---

# memory_growing

## Signature

```rust
fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool>
```

## Source
Lines 17–25 in `crates/frappe-framework/src/wasmtime_sandbox.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [wasmtime_sandbox](/crates/frappe-framework/src/wasmtime_sandbox.md) |
