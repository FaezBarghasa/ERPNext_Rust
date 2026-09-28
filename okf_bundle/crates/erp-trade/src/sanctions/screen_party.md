---
okf_version: "0.2"
type: Function
title: screen_party
description: "Screens an entity against the watchlist. Returns matched entries if similarity >= threshold (default 0.85)."
resource: crates/erp-trade/src/sanctions.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-trade"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:26:15Z"
concept_id: crates/erp-trade/src/sanctions/screen_party
language: rust
---

# screen_party

Screens an entity against the watchlist. Returns matched entries if similarity >= threshold (default 0.85).

## Signature

```rust
impl SanctionsScreener { pub fn screen_party(
        party_name: &str,
        watchlist: &'a [SanctionEntry],
        threshold: f64,
    ) -> Vec<(&'a SanctionEntry, f64)> }
```

## Type Parameters

- `'a`

## Visibility

- `pub`

## Docstring

Screens an entity against the watchlist. Returns matched entries if similarity >= threshold (default 0.85).

## Source
Lines 45–68 in `crates/erp-trade/src/sanctions.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [sanctions](/crates/erp-trade/src/sanctions.md) |
