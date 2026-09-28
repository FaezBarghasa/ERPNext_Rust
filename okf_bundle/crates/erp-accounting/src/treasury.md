---
okf_version: "0.2"
type: Module
title: treasury
description: "Automated Zero-Balance Account (ZBA) Cash Pooling & Intercompany Loans."
resource: crates/erp-accounting/src/treasury.rs
tags:
  - "lang:rust"
  - "type:Module"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-accounting/src/treasury
language: rust
---

# treasury

Automated Zero-Balance Account (ZBA) Cash Pooling & Intercompany Loans.

## Docstring

Automated Zero-Balance Account (ZBA) Cash Pooling & Intercompany Loans.

## Relationships

| Type | Target |
|------|--------|
| related | [BankAccount](/crates/erp-accounting/src/treasury/BankAccount.md) |
| related | [ZbaSweepTransaction](/crates/erp-accounting/src/treasury/ZbaSweepTransaction.md) |
| related | [TreasuryPoolingEngine](/crates/erp-accounting/src/treasury/TreasuryPoolingEngine.md) |
| related | [compute_eod_sweeps](/crates/erp-accounting/src/treasury/compute_eod_sweeps.md) |
| related | [daily_interest_accrual](/crates/erp-accounting/src/treasury/daily_interest_accrual.md) |
| related | [compute_eod_sweeps](/crates/erp-accounting/src/treasury/compute_eod_sweeps.md) |
| related | [daily_interest_accrual](/crates/erp-accounting/src/treasury/daily_interest_accrual.md) |
| related | [test_zba_cash_sweeps_and_interest](/crates/erp-accounting/src/treasury/test_zba_cash_sweeps_and_interest.md) |
| related | [rust_decimal](/_dependencies/cargo/rust_decimal.md) |
| related | [serde](/_dependencies/cargo/serde.md) |
| related | [rust_decimal_macros](/_dependencies/cargo/rust_decimal_macros.md) |
