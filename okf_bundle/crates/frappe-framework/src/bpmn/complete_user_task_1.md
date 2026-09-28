---
okf_version: "0.2"
type: Function
title: complete_user_task
description: User completes a pending user task.
resource: crates/frappe-framework/src/bpmn.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/bpmn/complete_user_task_1
language: rust
---

# complete_user_task

User completes a pending user task.

## Signature

```rust
pub fn complete_user_task(
        def: &BpmnProcessDefinition,
        instance: &mut ProcessInstance,
        task_id: &str,
    ) -> Result<(), String>
```

## Visibility

- `pub`

## Docstring

User completes a pending user task.

## Source
Lines 146–167 in `crates/frappe-framework/src/bpmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bpmn](/crates/frappe-framework/src/bpmn.md) |
