---
okf_version: "0.2"
type: Function
title: visible_slice
description: "Visible-row slice for virtualized grid (Stage 6.1.1): 60fps over 1M rows."
resource: crates/desk-components/src/grid.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/desk-components/src/grid/visible_slice
language: rust
---

# visible_slice

Visible-row slice for virtualized grid (Stage 6.1.1): 60fps over 1M rows.

## Signature

```rust
pub fn visible_slice(
    total: usize,
    row_h: usize,
    scroll: usize,
    viewport_h: usize,
) -> (usize, usize)
```

## Visibility

- `pub`

## Docstring

Visible-row slice for virtualized grid (Stage 6.1.1): 60fps over 1M rows.

## Source
Lines 2–14 in `crates/desk-components/src/grid.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [grid](/crates/desk-components/src/grid.md) |
| called_by | [main](/crates/desk-app/src/main/main.md) |
