---
okf_version: "0.2"
type: Class
title: ProcessInstance
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/frappe-framework/src/bpmn.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/bpmn/ProcessInstance
language: rust
---

# ProcessInstance

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct ProcessInstance
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `id`
- `process_id`
- `active_tokens`
- `completed_nodes`
- `variables`
- `is_completed`

## Source
Lines 42–49 in `crates/frappe-framework/src/bpmn.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [bpmn](/crates/frappe-framework/src/bpmn.md) |
