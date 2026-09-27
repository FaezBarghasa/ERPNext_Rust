# Changelog

All notable changes to the ERPNext Rust Workspace will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-09-27

### Added
- **Core Framework & Storage**:
  - `frappe-meta`: Schema compiler, RBAC permission validator, dynamic naming series, and migration manager.
  - `frappe-framework`: Document lifecycle engine and sandboxed execution runtimes (Rhai, Wasmtime).
  - `frappe-storage`: Deduplicated content-addressable storage engine and SurrealDB integration layer.
  - `frappe-net`: Multi-tenant HTTP/WS routing, live sync event streaming, and async background queue workers.
- **Enterprise Modules**:
  - `erp-accounting`: Immutable double-entry general ledger, multi-currency conversion, asset depreciation, and AR/AP aging.
  - `erp-inventory`: Real-time stock balance tracking, FIFO queue valuation, and batch/warehouse management.
  - `erp-manufacturing`: Multilevel BOM explosion, routing, and MRP calculation engines.
  - `erp-trade`: Pricing rules, multi-tier discount logic, landed cost allocation, and dynamic tax calculation.
  - `erp-hr`: Employee management, attendance tracking, and automated payroll processing.
  - `erp-crm`: Lead capture pipelines, deal tracking, and rule-based opportunity scoring.
  - `erp-lending`: Amortization schedule computation and loan interest accruals.
  - `erp-support`: SLA tracking, priority matrices, and customer support gameplans.
  - `erp-learning`: Course curriculum management, student enrollment, and quiz evaluations.
  - `erp-cms`: Token-authenticated secure media streaming, timestamped subtitle parser, and transcoder integration.
- **Frontend & Benchmarking**:
  - `desk-components`: Reusable UI components, dynamic JSON-schema form engine, video HUD overlays, and reactive signal abstractions.
  - `desk-app`: Application desktop/web client shell.
  - `rbench`: High-throughput synthetic benchmark generator.
- **Documentation**:
  - Comprehensive workspace README, Architecture Specification (`docs/architecture.md`), Product Requirements Document (`docs/prd.md`), and TDD Guide (`docs/tdd.md`).
