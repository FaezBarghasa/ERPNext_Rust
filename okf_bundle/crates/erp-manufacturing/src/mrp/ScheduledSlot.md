---
okf_version: "0.2"
type: Class
title: ScheduledSlot
description: Scheduled operational time window on a workstation.
resource: crates/erp-manufacturing/src/mrp.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:27:43Z"
concept_id: crates/erp-manufacturing/src/mrp/ScheduledSlot
language: rust
---

# ScheduledSlot

Scheduled operational time window on a workstation.

## Signature

```rust
pub struct ScheduledSlot
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Scheduled operational time window on a workstation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `job_id`
- `start_time`
- `end_time`

## Source
Lines 74–81 in `crates/erp-manufacturing/src/mrp.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [mrp](/crates/erp-manufacturing/src/mrp.md) |
