---
okf_version: "0.2"
type: Class
title: WorkspaceTask
description: Collaborative project task.
resource: crates/erp-support/src/gameplan.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:44:55Z"
concept_id: crates/erp-support/src/gameplan/WorkspaceTask
language: rust
---

# WorkspaceTask

Collaborative project task.

## Signature

```rust
pub struct WorkspaceTask
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Collaborative project task.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `project_id`
- `title`
- `status`
- `assigned_user`

## Source
Lines 15–21 in `crates/erp-support/src/gameplan.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [gameplan](/crates/erp-support/src/gameplan.md) |
