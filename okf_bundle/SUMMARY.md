---
description: 'Top-level OKF summary: 1215 concepts across 3 domains and 66 modules'
git_branch: main
git_repo: ERPNext_Rust
okf_version: '0.2'
timestamp: '2026-09-27T20:15:55Z'
title: ERPNext_workspace — Knowledge Summary
type: Index
---

# ERPNext_workspace — Knowledge Summary

> OKF v0.2 bundle | 1,215 concepts | 3 domains | 66 modules

## Stats

| Type | Count |
|------|-------|
| Dependency | 717 |
| Function | 302 |
| Class | 129 |
| Module | 66 |
| Resource | 1 |

| Language | Concepts |
|----------|----------|
| manifest | 717 |
| rust | 496 |
| yaml | 2 |

## Domain Map

Use these links to navigate the bundle or prime an AI agent with focused context.

### [Taskfile.yaml](Taskfile.yaml/index.md) — 2 concepts

- [Taskfile](Taskfile/index.md) (2 concepts) — YAML file: Taskfile.yaml (1 document(s))

### [benches](benches/index.md) — 1 concepts

- [benches/tenant_bench](benches/tenant_bench/index.md) (1 concepts)

### [crates](crates/index.md) — 495 concepts

- [crates/frappe-net/src/tenant](crates/frappe-net/src/tenant/index.md) (29 concepts)
- [crates/frappe-framework/src/lifecycle](crates/frappe-framework/src/lifecycle/index.md) (24 concepts)
- [crates/erp-accounting/src/ledger](crates/erp-accounting/src/ledger/index.md) (23 concepts)
- [crates/frappe-storage/src/lib](crates/frappe-storage/src/lib/index.md) (20 concepts) — Tenant context, queue states, CAS deduplication, and SurrealDB storage backend.
- [crates/erp-cms/src/security](crates/erp-cms/src/security/index.md) (18 concepts)
- [crates/erp-learning/src/lms](crates/erp-learning/src/lms/index.md) (17 concepts)
- [crates/erp-hr/src/payroll](crates/erp-hr/src/payroll/index.md) (16 concepts)
- [crates/erp-lending/src/amortization](crates/erp-lending/src/amortization/index.md) (15 concepts)
- *…and 56 more modules*

## Dependencies

> Full list at [`_dependencies/index.md`](/_dependencies/index.md) or `okf lookup --type Dependency`

| Ecosystem | Packages |
|----------|----------|
| cargo | 713 |
| docker | 3 |
| docker-compose | 1 |

## Key Concepts

Highest-value concepts across all domains (Classes and Functions with rich descriptions).

| Concept | Type | Module | Description |
|---------|------|--------|-------------|
| [verify_balance_sheet](/crates/erp-accounting/src/ledger/verify_balance_sheet.md) | Function | `crates/erp-accounting/src` | Verifies the Fundamental Accounting Equation: $\text{Assets}… |
| [verify_balance_sheet](/crates/erp-accounting/src/ledger/verify_balance_sheet_1.md) | Function | `crates/erp-accounting/src` | Verifies the Fundamental Accounting Equation: $\text{Assets}… |
| [verify_balanced_dec](/crates/erp-accounting/src/decimal_ledger/verify_balanced_dec.md) | Function | `crates/erp-accounting/src` | [derive(Debug, Clone)] pub struct DecLine { pub account: Str… |
| [compile_to_surrealql](/crates/frappe-meta/src/schema_compiler/compile_to_surrealql.md) | Function | `crates/frappe-meta/src` | Compiles a strongly typed `DocTypeSchema` into a sequence of… |
| [evaluate_tickets](/crates/erp-support/src/sla/evaluate_tickets.md) | Function | `crates/erp-support/src` | Evaluates unresolved tickets against their SLA response dead… |
| [evaluate_tickets](/crates/erp-support/src/sla/evaluate_tickets_1.md) | Function | `crates/erp-support/src` | Evaluates unresolved tickets against their SLA response dead… |
| [balance_exchange_variance](/crates/erp-accounting/src/ledger/balance_exchange_variance.md) | Function | `crates/erp-accounting/src` | Automatically generates balancing Exchange Gain/Loss line wh… |
| [balance_exchange_variance](/crates/erp-accounting/src/ledger/balance_exchange_variance_1.md) | Function | `crates/erp-accounting/src` | Automatically generates balancing Exchange Gain/Loss line wh… |
| [process_batch_payroll](/crates/erp-hr/src/payroll/process_batch_payroll.md) | Function | `crates/erp-hr/src` | Runs concurrent payroll calculation across workforce using T… |
| [process_batch_payroll](/crates/erp-hr/src/payroll/process_batch_payroll_1.md) | Function | `crates/erp-hr/src` | Runs concurrent payroll calculation across workforce using T… |
| [insert](/crates/frappe-framework/src/lifecycle/insert.md) | Function | `crates/frappe-framework/src` | Handles document insertion: runs naming series, validation h… |
| [insert](/crates/frappe-framework/src/lifecycle/insert_1.md) | Function | `crates/frappe-framework/src` | Handles document insertion: runs naming series, validation h… |
| [generate_schedule](/crates/erp-lending/src/amortization/generate_schedule.md) | Function | `crates/erp-lending/src` | Generates complete amortization schedule with zero-loss fina… |
| [generate_schedule](/crates/erp-lending/src/amortization/generate_schedule_1.md) | Function | `crates/erp-lending/src` | Generates complete amortization schedule with zero-loss fina… |
| [evaluate_and_issue_certificate](/crates/erp-learning/src/lms/evaluate_and_issue_certificate.md) | Function | `crates/erp-learning/src` | Evaluates assessment and auto-issues certificate if 100% com… |
| [evaluate_and_issue_certificate](/crates/erp-learning/src/lms/evaluate_and_issue_certificate_1.md) | Function | `crates/erp-learning/src` | Evaluates assessment and auto-issues certificate if 100% com… |
| [allocate_fifo](/crates/erp-accounting/src/receivables/allocate_fifo.md) | Function | `crates/erp-accounting/src` | FIFO Payment Allocation: matches incoming payment against ol… |
| [allocate_fifo](/crates/erp-accounting/src/receivables/allocate_fifo_1.md) | Function | `crates/erp-accounting/src` | FIFO Payment Allocation: matches incoming payment against ol… |
| [DeduplicatedStorage](/crates/frappe-storage/src/drive/DeduplicatedStorage.md) | Class | `crates/frappe-storage/src` | Content-Addressable Storage (CAS) with SHA-256 deduplication… |
| [cosine_similarity](/crates/erp-cms/src/subtitles/cosine_similarity.md) | Function | `crates/erp-cms/src` | Computes cosine similarity between two vector embeddings: $\… |

## Usage with OpenCode

```bash
# Prime full context
RUN cat ./okf_bundle/SUMMARY.md

# Prime specific domain
RUN cat ./okf_bundle/Taskfile.yaml/index.md

# Find a concept
RUN find ./okf_bundle -name '<ConceptName>.md' | xargs cat
```
