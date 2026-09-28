---
okf_version: "0.2"
type: Function
title: sign_record
resource: crates/erp-manufacturing/src/ebr.rs
tags:
  - "lang:rust"
  - "type:Function"
  - "module:crates"
  - "domain:erp-manufacturing"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:49:30Z"
concept_id: crates/erp-manufacturing/src/ebr/sign_record
language: rust
---

# sign_record

## Signature

```rust
impl ElectronicBatchRecord { fn sign_record(
        user_id: &str,
        name: &str,
        meaning: &str,
        data: &serde_json::Value,
        secret: &str,
    ) -> WitnessSignature }
```

## Source
Lines 86–113 in `crates/erp-manufacturing/src/ebr.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [ebr](/crates/erp-manufacturing/src/ebr.md) |
