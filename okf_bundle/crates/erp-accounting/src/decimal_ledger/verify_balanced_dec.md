---
okf_version: "0.2"
type: Function
title: verify_balanced_dec
description: Zero-balance invariant with exact decimal math.
resource: crates/erp-accounting/src/decimal_ledger.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/decimal_ledger/verify_balanced_dec
language: rust
---

# verify_balanced_dec

Zero-balance invariant with exact decimal math.

## Signature

```rust
pub fn verify_balanced_dec(lines: &[DecLine]) -> bool
```

## Visibility

- `pub`

## Docstring

Zero-balance invariant with exact decimal math.

## Source
Lines 9–14 in `crates/erp-accounting/src/decimal_ledger.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [decimal_ledger](/crates/erp-accounting/src/decimal_ledger.md) |
