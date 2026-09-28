---
okf_version: "0.2"
type: Class
title: Lead
description: CRM Lead record.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:56:26Z"
concept_id: crates/erp-crm/src/pipeline/Lead
language: rust
---

# Lead

CRM Lead record.

## Signature

```rust
pub struct Lead
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

CRM Lead record.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `lead_name`
- `email`
- `status`

## Source
Lines 51–56 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
