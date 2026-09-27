---
okf_version: "0.2"
type: Function
title: verify_balanced_dec
description: "[derive(Debug, Clone)] pub struct DecLine { pub account: String, pub debit: Decimal, pub credit: Decimal }"
resource: crates/erp-accounting/src/decimal_ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T15:20:48Z"
concept_id: crates/erp-accounting/src/decimal_ledger/verify_balanced_dec
language: rust
---

# verify_balanced_dec

[derive(Debug, Clone)] pub struct DecLine { pub account: String, pub debit: Decimal, pub credit: Decimal }

## Signature

```rust
pub fn verify_balanced_dec(lines: &[DecLine]) -> bool
```

## Visibility

- `pub`

## Docstring

[derive(Debug, Clone)] pub struct DecLine { pub account: String, pub debit: Decimal, pub credit: Decimal }
Zero-balance invariant with exact decimal math.

## Source
Lines 4–6 in `crates/erp-accounting/src/decimal_ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [decimal_ledger](/crates/erp-accounting/src/decimal_ledger.md) |
