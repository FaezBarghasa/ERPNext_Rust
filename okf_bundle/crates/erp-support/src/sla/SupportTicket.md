---
okf_version: "0.2"
type: Class
title: SupportTicket
description: Support Ticket record.
resource: crates/erp-support/src/sla.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-support"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:44:16Z"
concept_id: crates/erp-support/src/sla/SupportTicket
language: rust
---

# SupportTicket

Support Ticket record.

## Signature

```rust
pub struct SupportTicket
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Support Ticket record.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `id`
- `customer`
- `subject`
- `priority`
- `status`
- `assigned_to`
- `created_at`
- `response_deadline`

## Source
Lines 36–45 in `crates/erp-support/src/sla.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sla](/crates/erp-support/src/sla.md) |
