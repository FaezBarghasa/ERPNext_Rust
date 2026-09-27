---
okf_version: "0.2"
type: Function
title: sample_accounts
resource: crates/erp-accounting/src/lib.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:11:46Z"
concept_id: crates/erp-accounting/src/lib/sample_accounts
language: rust
---

# sample_accounts

## Signature

```rust
fn sample_accounts() -> Vec<Account>
```

## Source
Lines 23–66 in `crates/erp-accounting/src/lib.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lib](/crates/erp-accounting/src/lib.md) |
| called_by | [test_balance_sheet_equation](/crates/erp-accounting/src/lib/test_balance_sheet_equation.md) |
| called_by | [test_period_closed_rejection](/crates/erp-accounting/src/lib/test_period_closed_rejection.md) |
