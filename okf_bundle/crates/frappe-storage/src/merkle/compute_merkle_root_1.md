---
okf_version: "0.2"
type: Function
title: compute_merkle_root
description: Computes root hash of a list of binary leaves.
resource: crates/frappe-storage/src/merkle.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/merkle/compute_merkle_root_1
language: rust
---

# compute_merkle_root

Computes root hash of a list of binary leaves.

## Signature

```rust
pub fn compute_merkle_root(leaves: &[String]) -> String
```

## Decorators

- `must_use`

## Visibility

- `pub`

## Docstring

Computes root hash of a list of binary leaves.
[must_use]

## Source
Lines 38–59 in `crates/frappe-storage/src/merkle.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [merkle](/crates/frappe-storage/src/merkle.md) |
