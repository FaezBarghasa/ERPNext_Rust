---
okf_version: "0.2"
type: Class
title: Document
description: In-memory representation of a Frappe document instance.
resource: crates/frappe-framework/src/lifecycle.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:frappe-framework"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-framework/src/lifecycle/Document
language: rust
---

# Document

In-memory representation of a Frappe document instance.

## Signature

```rust
pub struct Document
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq)`

## Visibility

- `pub`

## Docstring

In-memory representation of a Frappe document instance.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

## Methods

- `name`
- `doctype`
- `docstatus`
- `data`

## Source
Lines 31–40 in `crates/frappe-framework/src/lifecycle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [lifecycle](/crates/frappe-framework/src/lifecycle.md) |
