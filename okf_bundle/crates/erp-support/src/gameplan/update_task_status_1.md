---
okf_version: "0.2"
type: Function
title: update_task_status
description: Mutates task status.
resource: crates/erp-support/src/gameplan.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:44:55Z"
concept_id: crates/erp-support/src/gameplan/update_task_status_1
language: rust
---

# update_task_status

Mutates task status.

## Signature

```rust
pub fn update_task_status(
        &mut self,
        task_id: &str,
        new_status: TaskStatus,
    ) -> Option<WorkspaceTask>
```

## Visibility

- `pub`

## Docstring

Mutates task status.

## Source
Lines 37–48 in `crates/erp-support/src/gameplan.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [gameplan](/crates/erp-support/src/gameplan.md) |
