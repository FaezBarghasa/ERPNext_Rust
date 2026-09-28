---
okf_version: "0.2"
type: Function
title: diff_tokens
description: Compares standard clause tokens against inbound third-party legal redlines.
resource: crates/erp-crm/src/clm.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-crm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-crm/src/clm/diff_tokens_1
language: rust
---

# diff_tokens

Compares standard clause tokens against inbound third-party legal redlines.

## Signature

```rust
pub fn diff_tokens(standard: &str, inbound: &str) -> (Vec<String>, Vec<String>)
```

## Visibility

- `pub`

## Docstring

Compares standard clause tokens against inbound third-party legal redlines.

## Source
Lines 26–43 in `crates/erp-crm/src/clm.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [clm](/crates/erp-crm/src/clm.md) |
