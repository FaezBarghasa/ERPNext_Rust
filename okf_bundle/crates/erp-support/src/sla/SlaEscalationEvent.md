---
okf_version: "0.2"
type: Class
title: SlaEscalationEvent
description: Escalation event generated upon SLA breach.
resource: crates/erp-support/src/sla.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:43:11Z"
concept_id: crates/erp-support/src/sla/SlaEscalationEvent
language: rust
---

# SlaEscalationEvent

Escalation event generated upon SLA breach.

## Signature

```rust
pub struct SlaEscalationEvent
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Escalation event generated upon SLA breach.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `ticket_id`
- `previous_priority`
- `new_priority`
- `breached_at`

## Source
Lines 84–89 in `crates/erp-support/src/sla.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sla](/crates/erp-support/src/sla.md) |
