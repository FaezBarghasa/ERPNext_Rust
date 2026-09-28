---
okf_version: "0.2"
type: Function
title: transition
resource: crates/frappe-storage/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/lib/transition
language: rust
---

# transition

## Signature

```rust
impl QueueTask { pub fn transition(&mut self, to: TaskState) -> bool }
```

## Visibility

- `pub`

## Source
Lines 82–94 in `crates/frappe-storage/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/frappe-storage/src/lib.md) |
