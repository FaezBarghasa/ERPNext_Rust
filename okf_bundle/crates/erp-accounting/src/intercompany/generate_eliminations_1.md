---
okf_version: "0.2"
type: Function
title: generate_eliminations
description: Generates GAAP/IFRS consolidation eliminations for intercompany sales and inventory profit.
resource: crates/erp-accounting/src/intercompany.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-accounting"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:22:07Z"
concept_id: crates/erp-accounting/src/intercompany/generate_eliminations_1
language: rust
---

# generate_eliminations

Generates GAAP/IFRS consolidation eliminations for intercompany sales and inventory profit.

## Signature

```rust
pub fn generate_eliminations(
        txs: &[IntercompanyTransaction],
        remaining_inventory_ratio: Decimal, // % of intercompany goods still in buyer warehouse
    ) -> Vec<EliminationJournalEntry>
```

## Visibility

- `pub`

## Docstring

Generates GAAP/IFRS consolidation eliminations for intercompany sales and inventory profit.

## Source
Lines 29–63 in `crates/erp-accounting/src/intercompany.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [intercompany](/crates/erp-accounting/src/intercompany.md) |
