---
okf_version: "0.2"
type: Class
title: SupportAgent
description: Support technician agent representation.
resource: crates/erp-support/src/sla.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:43:11Z"
concept_id: crates/erp-support/src/sla/SupportAgent
language: rust
---

# SupportAgent

Support technician agent representation.

## Signature

```rust
pub struct SupportAgent
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Support technician agent representation.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `agent_id`
- `name`
- `is_online`
- `active_ticket_count`

## Source
Lines 49–54 in `crates/erp-support/src/sla.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sla](/crates/erp-support/src/sla.md) |
