---
okf_version: "0.2"
type: Function
title: step
description: "Advances execution tokens until a UserTask, EndEvent, or wait state is reached."
resource: crates/frappe-framework/src/bpmn.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/bpmn/step
language: rust
---

# step

Advances execution tokens until a UserTask, EndEvent, or wait state is reached.

## Signature

```rust
impl BpmnEngine { pub fn step(
        def: &BpmnProcessDefinition,
        instance: &mut ProcessInstance,
    ) -> Result<Vec<String>, String> }
```

## Visibility

- `pub`

## Docstring

Advances execution tokens until a UserTask, EndEvent, or wait state is reached.

## Source
Lines 70–143 in `crates/frappe-framework/src/bpmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bpmn](/crates/frappe-framework/src/bpmn.md) |
