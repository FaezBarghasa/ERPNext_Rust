---
description: 'Top-level OKF summary: 1859 concepts across 3 domains and 110 modules'
git_branch: main
git_repo: ERPNext_Rust
okf_version: '0.2'
timestamp: '2026-09-27T22:05:46Z'
title: ERPNext_workspace — Knowledge Summary
type: Index
---

# ERPNext_workspace — Knowledge Summary

> OKF v0.2 bundle | 1,859 concepts | 3 domains | 110 modules

## Stats

| Type | Count |
|------|-------|
| Dependency | 893 |
| Function | 586 |
| Class | 269 |
| Module | 110 |
| Resource | 1 |

| Language | Concepts |
|----------|----------|
| rust | 964 |
| manifest | 893 |
| yaml | 2 |

## Domain Map

Use these links to navigate the bundle or prime an AI agent with focused context.

### [Taskfile.yaml](Taskfile.yaml/index.md) — 2 concepts

- [Taskfile](Taskfile/index.md) (2 concepts) — YAML file: Taskfile.yaml (1 document(s))

### [benches](benches/index.md) — 1 concepts

- [benches/tenant_bench](benches/tenant_bench/index.md) (1 concepts)

### [crates](crates/index.md) — 963 concepts

- [crates/frappe-net/src/tenant](crates/frappe-net/src/tenant/index.md) (29 concepts)
- [crates/frappe-framework/src/lifecycle](crates/frappe-framework/src/lifecycle/index.md) (24 concepts)
- [crates/erp-accounting/src/ledger](crates/erp-accounting/src/ledger/index.md) (23 concepts)
- [crates/frappe-storage/src/lib](crates/frappe-storage/src/lib/index.md) (20 concepts)
- [crates/frappe-storage/src/bitemporal](crates/frappe-storage/src/bitemporal/index.md) (19 concepts) — Bi-temporal state ledger supporting system time (transaction time) and valid tim
- [crates/frappe-framework/src/saga](crates/frappe-framework/src/saga/index.md) (18 concepts) — Distributed Saga Orchestration Coordinator with forward execution and compensati
- [crates/erp-ppm/src/construction](crates/erp-ppm/src/construction/index.md) (18 concepts) — EPC Contract Administration, AIA G702/G703 Billing, Retainage & Change Orders.
- [crates/erp-cms/src/security](crates/erp-cms/src/security/index.md) (18 concepts)
- *…and 100 more modules*

## Dependencies

> Full list at [`_dependencies/index.md`](/_dependencies/index.md) or `okf lookup --type Dependency`

| Ecosystem | Packages |
|----------|----------|
| cargo | 889 |
| docker | 3 |
| docker-compose | 1 |

## Key Concepts

Highest-value concepts across all domains (Classes and Functions with rich descriptions).

| Concept | Type | Module | Description |
|---------|------|--------|-------------|
| [verify_balance_sheet](/crates/erp-accounting/src/ledger/verify_balance_sheet.md) | Function | `crates/erp-accounting/src` | Verifies the Fundamental Accounting Equation: $\text{Assets}… |
| [verify_balance_sheet](/crates/erp-accounting/src/ledger/verify_balance_sheet_1.md) | Function | `crates/erp-accounting/src` | Verifies the Fundamental Accounting Equation: $\text{Assets}… |
| [allocate_contract_price](/crates/erp-crm/src/revops/allocate_contract_price.md) | Function | `crates/erp-crm/src` | 5-Step Revenue Recognition: Allocates Transaction Price (TP)… |
| [allocate_contract_price](/crates/erp-crm/src/revops/allocate_contract_price_1.md) | Function | `crates/erp-crm/src` | 5-Step Revenue Recognition: Allocates Transaction Price (TP)… |
| [screen_party](/crates/erp-trade/src/sanctions/screen_party.md) | Function | `crates/erp-trade/src` | Screens an entity against the watchlist. Returns matched ent… |
| [screen_party](/crates/erp-trade/src/sanctions/screen_party_1.md) | Function | `crates/erp-trade/src` | Screens an entity against the watchlist. Returns matched ent… |
| [recommended_safety_stock](/crates/erp-asset/src/mro/recommended_safety_stock.md) | Function | `crates/erp-asset/src` | Calculates required safety stock quantity S such that cumula… |
| [recommended_safety_stock](/crates/erp-asset/src/mro/recommended_safety_stock_1.md) | Function | `crates/erp-asset/src` | Calculates required safety stock quantity S such that cumula… |
| [authorize](/crates/erp-asset/src/safety/authorize.md) | Function | `crates/erp-asset/src` | Validates all cryptographic and physical isolation prerequis… |
| [authorize](/crates/erp-asset/src/safety/authorize_1.md) | Function | `crates/erp-asset/src` | Validates all cryptographic and physical isolation prerequis… |
| [compile_to_surrealql](/crates/frappe-meta/src/schema_compiler/compile_to_surrealql.md) | Function | `crates/frappe-meta/src` | Compiles a strongly typed `DocTypeSchema` into a sequence of… |
| [compute_eod_sweeps](/crates/erp-accounting/src/treasury/compute_eod_sweeps.md) | Function | `crates/erp-accounting/src` | Evaluates end-of-day subsidiary accounts and sweeps excess c… |
| [compute_eod_sweeps](/crates/erp-accounting/src/treasury/compute_eod_sweeps_1.md) | Function | `crates/erp-accounting/src` | Evaluates end-of-day subsidiary accounts and sweeps excess c… |
| [evaluate_tickets](/crates/erp-support/src/sla/evaluate_tickets.md) | Function | `crates/erp-support/src` | Evaluates unresolved tickets against their SLA response dead… |
| [evaluate_tickets](/crates/erp-support/src/sla/evaluate_tickets_1.md) | Function | `crates/erp-support/src` | Evaluates unresolved tickets against their SLA response dead… |
| [simulate](/crates/erp-ppm/src/monte_carlo/simulate.md) | Function | `crates/erp-ppm/src` | Evaluates stochastic task durations over N iterations using … |
| [simulate](/crates/erp-ppm/src/monte_carlo/simulate_1.md) | Function | `crates/erp-ppm/src` | Evaluates stochastic task durations over N iterations using … |
| [balance_exchange_variance](/crates/erp-accounting/src/ledger/balance_exchange_variance.md) | Function | `crates/erp-accounting/src` | Automatically generates balancing Exchange Gain/Loss line wh… |
| [balance_exchange_variance](/crates/erp-accounting/src/ledger/balance_exchange_variance_1.md) | Function | `crates/erp-accounting/src` | Automatically generates balancing Exchange Gain/Loss line wh… |
| [process_batch_payroll](/crates/erp-hr/src/payroll/process_batch_payroll.md) | Function | `crates/erp-hr/src` | Runs concurrent payroll calculation across workforce using T… |

## Usage with OpenCode

```bash
# Prime full context
RUN cat ./okf_bundle/SUMMARY.md

# Prime specific domain
RUN cat ./okf_bundle/Taskfile.yaml/index.md

# Find a concept
RUN find ./okf_bundle -name '<ConceptName>.md' | xargs cat
```
