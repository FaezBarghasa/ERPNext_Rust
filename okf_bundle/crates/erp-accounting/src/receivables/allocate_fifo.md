---
okf_version: "0.2"
type: Function
title: allocate_fifo
description: "FIFO Payment Allocation: matches incoming payment against oldest open invoices sequentially."
resource: crates/erp-accounting/src/receivables.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:10:14Z"
concept_id: crates/erp-accounting/src/receivables/allocate_fifo
language: rust
---

# allocate_fifo

FIFO Payment Allocation: matches incoming payment against oldest open invoices sequentially.

## Signature

```rust
impl ArApEngine { pub fn allocate_fifo(
        invoices: &mut [OpenInvoice],
        mut payment_amount: Decimal,
    ) -> (Vec<PaymentAllocationResult>, Decimal) }
```

## Visibility

- `pub`

## Docstring

FIFO Payment Allocation: matches incoming payment against oldest open invoices sequentially.
Returns allocation details and unallocated remaining payment amount.

## Source
Lines 36–74 in `crates/erp-accounting/src/receivables.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [receivables](/crates/erp-accounting/src/receivables.md) |
