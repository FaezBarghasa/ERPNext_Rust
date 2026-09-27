---
okf_version: "0.2"
type: Class
title: Account
description: Chart of Accounts node record.
resource: crates/erp-accounting/src/coa.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:09:10Z"
concept_id: crates/erp-accounting/src/coa/Account
language: rust
---

# Account

Chart of Accounts node record.

## Signature

```rust
pub struct Account
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Chart of Accounts node record.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `name`
- `account_number`
- `parent_account`
- `is_group`
- `root_type`
- `account_currency`

## Source
Lines 20–33 in `crates/erp-accounting/src/coa.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [coa](/crates/erp-accounting/src/coa.md) |
