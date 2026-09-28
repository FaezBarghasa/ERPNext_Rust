---
okf_version: "0.2"
type: Function
title: eval_depends_on
description: "Evaluates Frappe dynamic form `depends_on` visibility conditions."
resource: crates/desk-components/src/forms.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:desk-components"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/desk-components/src/forms/eval_depends_on
language: rust
---

# eval_depends_on

Evaluates Frappe dynamic form `depends_on` visibility conditions.

## Signature

```rust
pub fn eval_depends_on(expr: &str, form_values: &HashMap<String, String>) -> bool
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Evaluates Frappe dynamic form `depends_on` visibility conditions.
[must_use]

## Source
Lines 139–158 in `crates/desk-components/src/forms.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [forms](/crates/desk-components/src/forms.md) |
