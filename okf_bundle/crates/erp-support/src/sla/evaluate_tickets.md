---
okf_version: "0.2"
type: Function
title: evaluate_tickets
description: Evaluates unresolved tickets against their SLA response deadlines and escalates breached tickets.
resource: crates/erp-support/src/sla.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:44:16Z"
concept_id: crates/erp-support/src/sla/evaluate_tickets
language: rust
---

# evaluate_tickets

Evaluates unresolved tickets against their SLA response deadlines and escalates breached tickets.

## Signature

```rust
impl SlaWatchdog { pub fn evaluate_tickets(
        tickets: &mut [SupportTicket],
        now: DateTime<Utc>,
    ) -> Vec<SlaEscalationEvent> }
```

## Visibility

- `pub`

## Docstring

Evaluates unresolved tickets against their SLA response deadlines and escalates breached tickets.
[must_use]

## Source
Lines 97–120 in `crates/erp-support/src/sla.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sla](/crates/erp-support/src/sla.md) |
