---
okf_version: "0.2"
type: Class
title: Dependency
description: "[derive(Clone, Debug, Serialize, Deserialize)]"
resource: crates/erp-ppm/src/scheduler.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-ppm"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/erp-ppm/src/scheduler/Dependency
language: rust
---

# Dependency

[derive(Clone, Debug, Serialize, Deserialize)]

## Signature

```rust
pub struct Dependency
```

## Decorators

- `derive(Clone, Debug, Serialize, Deserialize)`

## Visibility

- `pub`

## Docstring

[derive(Clone, Debug, Serialize, Deserialize)]

## Methods

- `predecessor_id`
- `dep_type`
- `lag`

## Source
Lines 14–18 in `crates/erp-ppm/src/scheduler.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [scheduler](/crates/erp-ppm/src/scheduler.md) |
