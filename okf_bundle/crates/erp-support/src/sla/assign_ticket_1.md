---
okf_version: "0.2"
type: Function
title: assign_ticket
description: Assigns an incoming ticket to the online technician with the lowest active workload.
resource: crates/erp-support/src/sla.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:44:16Z"
concept_id: crates/erp-support/src/sla/assign_ticket_1
language: rust
---

# assign_ticket

Assigns an incoming ticket to the online technician with the lowest active workload.

## Signature

```rust
pub fn assign_ticket(
        ticket: &mut SupportTicket,
        agents: &'a mut [SupportAgent],
    ) -> Result<&'a mut SupportAgent, SupportError>
```

## Type Parameters

- `'a`

## Visibility

- `pub`

## Docstring

Assigns an incoming ticket to the online technician with the lowest active workload.

## Source
Lines 61–79 in `crates/erp-support/src/sla.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sla](/crates/erp-support/src/sla.md) |
