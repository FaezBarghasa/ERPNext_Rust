---
okf_version: "0.2"
type: Function
title: validate_dispatch_from
description: Validates serial number location prior to dispatch.
resource: crates/erp-inventory/src/batches.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-inventory"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T19:17:29Z"
concept_id: crates/erp-inventory/src/batches/validate_dispatch_from_1
language: rust
---

# validate_dispatch_from

Validates serial number location prior to dispatch.

## Signature

```rust
pub fn validate_dispatch_from(
        &self,
        from_warehouse: &str,
    ) -> Result<(), InventoryError>
```

## Visibility

- `pub`

## Docstring

Validates serial number location prior to dispatch.

## Source
Lines 34–46 in `crates/erp-inventory/src/batches.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [batches](/crates/erp-inventory/src/batches.md) |
