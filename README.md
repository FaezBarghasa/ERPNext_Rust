# ERPNext Rust Workspace

Next-generation, high-performance, memory-safe rewrite of Frappe and ERPNext in Rust. Built for ultra-low latency, multi-tenant cloud-native deployments, and offline-first edge operations.

---

## 🏗 Workspace Architecture

The workspace is organized into modular crates across core infrastructure, enterprise business domain modules, desk UI components, and benchmarking tools:

```
ERPNext_workspace/
├── crates/
│   ├── frappe-meta/         # DocType schema engine, RBAC, naming series, migration runner
│   ├── frappe-framework/    # Document lifecycle hooks, Rhai/Wasmtime sandboxed scripting
│   ├── frappe-storage/      # SurrealDB storage backend, content-addressable deduplicated drive
│   ├── frappe-net/          # Multi-tenant HTTP/WS routing, queue worker, LiveSync channels
│   ├── erp-accounting/      # Double-entry ledger, multi-currency, GL, receivables, assets
│   ├── erp-inventory/       # Real-time stock ledgers, warehouse hierarchy, FIFO, batch tracking
│   ├── erp-manufacturing/   # Multilevel BOM explosions, work orders, MRP capacity planning
│   ├── erp-trade/           # Pricing rule engines, item taxes, landed cost allocations
│   ├── erp-hr/              # Employee lifecycle, attendance logs, salary structures, payroll
│   ├── erp-crm/             # Lead pipelines, opportunity scoring, conversion workflows
│   ├── erp-support/         # SLA management, escalation matrices, support gameplans
│   ├── erp-lending/         # Loan portfolios, interest accrual, amortization schedules
│   ├── erp-learning/        # Course syllabi, enrollment tracking, quiz assessment engines
│   ├── erp-cms/             # Token-authenticated media delivery, subtitles, transcoder
│   ├── desk-components/     # Cross-platform UI components (Dioxus), dynamic form engines
│   ├── desk-app/            # Desktop & Web entrypoint shell
│   └── rbench/              # High-throughput synthetic load generator and micro-benchmarks
└── docs/                    # Architectural specs, PRD, and TDD documentation
```

---

## 🚀 Key Features

- **Blazing Fast Performance**: Zero-overhead memory footprint with async runtime (Tokio/Actix).
- **Multi-Tenant Isolation**: Header and subdomain-based routing isolating database namespaces and storage securely.
- **Strict Accounting Integrity**: Complete immutable double-entry journal ledger with atomic multi-currency balance validation.
- **Sandboxed Extensibility**: Run client logic and automated workflows via Rhai and Wasmtime WASI runtimes.
- **Deduplicated Media & Storage**: Content-addressable storage with SHA-256 deduplication and signed token streaming.
- **Declarative Form Generation**: Schema-driven dynamic UI bindings for forms, HUD video overlays, and grid layouts.

---

## 🛠 Getting Started

### Prerequisites

- **Rust**: 1.85+ (2024 Edition / 2021 Edition compatible)
- **Cargo**: Standard toolchain

### Build & Test

```bash
# Verify compilation across all workspace crates
cargo check --workspace

# Run automated tests
cargo test --workspace

# Build optimized release binaries
cargo build --workspace --release
```

---

## 📚 Documentation

- [System Architecture](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/architecture.md)
- [Product Requirements Document (PRD)](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/prd.md)
- [Test-Driven Development (TDD) Guide](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/tdd.md)
- [Changelog](file:///home/jrad/RustroverProjects/ERPNext_workspace/CHANGELOG.md)
