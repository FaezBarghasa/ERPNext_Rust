---
okf_version: "0.2"
type: Class
title: Quotation
description: Commercial Quotation locking pricing and terms.
resource: crates/erp-crm/src/pipeline.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:56:26Z"
concept_id: crates/erp-crm/src/pipeline/Quotation
language: rust
---

# Quotation

Commercial Quotation locking pricing and terms.

## Signature

```rust
pub struct Quotation
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Commercial Quotation locking pricing and terms.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `party_name`
- `valid_till`
- `items`
- `net_total`
- `status`

## Source
Lines 93–100 in `crates/erp-crm/src/pipeline.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [pipeline](/crates/erp-crm/src/pipeline.md) |
