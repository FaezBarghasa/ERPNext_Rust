---
okf_version: "0.2"
type: Module
title: bitemporal
description: Bi-temporal state ledger supporting system time (transaction time) and valid time (business time).
resource: crates/frappe-storage/src/bitemporal.rs
tags:
  - "lang:rust"
  - "type:Module"
  - "module:crates"
  - "domain:frappe-storage"
  - "git:branch:main"
  - "git:repo:ERPNext_Rust"
timestamp: "2026-09-27T21:39:26Z"
concept_id: crates/frappe-storage/src/bitemporal
language: rust
---

# bitemporal

Bi-temporal state ledger supporting system time (transaction time) and valid time (business time).

## Docstring

Bi-temporal state ledger supporting system time (transaction time) and valid time (business time).

## Relationships

| Type | Target |
|------|--------|
| related | [TimeInterval](/crates/frappe-storage/src/bitemporal/TimeInterval.md) |
| related | [new](/crates/frappe-storage/src/bitemporal/new.md) |
| related | [point](/crates/frappe-storage/src/bitemporal/point.md) |
| related | [is_active_at](/crates/frappe-storage/src/bitemporal/is_active_at.md) |
| related | [new](/crates/frappe-storage/src/bitemporal/new.md) |
| related | [point](/crates/frappe-storage/src/bitemporal/point.md) |
| related | [is_active_at](/crates/frappe-storage/src/bitemporal/is_active_at.md) |
| related | [BiTemporalRecord](/crates/frappe-storage/src/bitemporal/BiTemporalRecord.md) |
| related | [new](/crates/frappe-storage/src/bitemporal/new.md) |
| related | [compute_hash](/crates/frappe-storage/src/bitemporal/compute_hash.md) |
| related | [verify_integrity](/crates/frappe-storage/src/bitemporal/verify_integrity.md) |
| related | [new](/crates/frappe-storage/src/bitemporal/new.md) |
| related | [compute_hash](/crates/frappe-storage/src/bitemporal/compute_hash.md) |
| related | [verify_integrity](/crates/frappe-storage/src/bitemporal/verify_integrity.md) |
| related | [BiTemporalQuery](/crates/frappe-storage/src/bitemporal/BiTemporalQuery.md) |
| related | [as_of](/crates/frappe-storage/src/bitemporal/as_of.md) |
| related | [as_of](/crates/frappe-storage/src/bitemporal/as_of.md) |
| related | [test_bitemporal_hashing_and_intervals](/crates/frappe-storage/src/bitemporal/test_bitemporal_hashing_and_intervals.md) |
| related | [chrono](/_dependencies/cargo/chrono.md) |
| related | [serde](/_dependencies/cargo/serde.md) |
