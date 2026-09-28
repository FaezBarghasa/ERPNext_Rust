---
okf_version: "0.2"
type: Function
title: render_svg_preview
description: Generates SVG markup for virtualized render.
resource: crates/desk-components/src/gantt.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/desk-components/src/gantt/render_svg_preview
language: rust
---

# render_svg_preview

Generates SVG markup for virtualized render.

## Signature

```rust
impl GanttViewModel { pub fn render_svg_preview(&self, row_height: f64, day_width: f64) -> String }
```

## Visibility

- `pub`

## Docstring

Generates SVG markup for virtualized render.
[must_use]

## Source
Lines 56–78 in `crates/desk-components/src/gantt.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [gantt](/crates/desk-components/src/gantt.md) |
