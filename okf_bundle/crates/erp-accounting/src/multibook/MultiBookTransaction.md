---
okf_version: "0.2"
type: Class
title: MultiBookTransaction
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-accounting/src/multibook.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/multibook/MultiBookTransaction
language: rust
---

# MultiBookTransaction

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct MultiBookTransaction
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `transaction_id`
- `posting_date`
- `company`
- `book`
- `lines`
- `memo`

## Source
Lines 23–30 in `crates/erp-accounting/src/multibook.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [multibook](/crates/erp-accounting/src/multibook.md) |
