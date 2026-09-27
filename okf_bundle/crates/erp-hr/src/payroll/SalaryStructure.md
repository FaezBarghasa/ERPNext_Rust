---
okf_version: "0.2"
type: Class
title: SalaryStructure
description: Employee salary structure configuration.
resource: crates/erp-hr/src/payroll.rs
tags:
  - "lang:rust"
  - "type:Class"
  - "module:crates"
  - "domain:erp-hr"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T20:27:26Z"
concept_id: crates/erp-hr/src/payroll/SalaryStructure
language: rust
---

# SalaryStructure

Employee salary structure configuration.

## Signature

```rust
pub struct SalaryStructure
```

## Decorators

- `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`

## Visibility

- `pub`

## Docstring

Employee salary structure configuration.
[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

## Methods

- `base_salary`
- `hra_percentage`
- `standard_deduction`
- `tax_withholding_rate`

## Source
Lines 11–20 in `crates/erp-hr/src/payroll.rs`

## Relationships

| Type | Target |
|------|--------|
| related | [payroll](/crates/erp-hr/src/payroll.md) |
