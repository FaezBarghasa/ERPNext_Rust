# Product Requirements Document (PRD)

## 1. Executive Summary

`rustnext` is an uncompromised, pure-Rust enterprise operating system and ERP substrate designed to replace legacy interpreted monolithic stacks. It delivers sub-millisecond transaction speeds, provable memory safety, local-first offline autonomy, and native multi-tenancy supporting thousands of isolated organizations per binary instance.

---

## 2. Quantitative System Goals & SLAs

| Requirement Category | Metric / Constraint | Target Objective |
| :--- | :--- | :--- |
| **Server-Side Rendering (SSR)** | Time to First Byte (TTFB) | $< 10\,\text{ms}$ on multi-block CMS pages |
| **Micro-Mode Topology** | Memory Footprint (RSS) | $\le 64\,\text{MB}$ in `--micro` profile |
| **WASI 0.2 Sandbox Trap** | Execution Time & Resource Ceiling | Traps loops in $\le 1.2\,\text{ms}$ with $32\,\text{MB}$ memory cap |
| **Accounting Balance Drift** | Double-Entry Invariant | Exactly $0.00\,\text{dec}$ ($\sum \text{Debit} = \sum \text{Credit}$) |
| **ZK Balance Sheet Proof** | Verification Time | $< 15\,\text{ms}$ non-interactive verification |
| **3-Way Invoice Matching** | Price & Quantity Variance | Automated approval within $\pm 0.5\%$ variance |
| **Local-First CRDT** | Convergence Guarantee | Strongly eventual consistency with zero loss |
| **3D WebGL Storefront Performance** | Sustained Frame Rate & Memory | $\ge 60\,\text{FPS}$ with $\le 18\,\text{MB}$ memory overhead |

---

## 3. Comprehensive Functional Requirements

### 3.1 Metadata & Schema Compilation (`frappe-meta`)
- **FR-META-01**: Must dynamically compile declarative `DocTypeSchema` structs into `SCHEMAFULL` SurrealQL DDL statements with index and relationship definitions.
- **FR-META-02**: Must provide zero-allocation `DynamicDocument` containers backed by `SmallVec<[_; 16]>` on the CPU stack.
- **FR-META-03**: Must support lock-free sequential naming series generators (e.g. `DRN-.YYYY.-.#####`).
- **FR-META-04**: Must provide an autonomous `AiSchemaSynthesizer` generating normalized DocTypes and Chart of Accounts from natural language prompts.
- **FR-META-05**: Must supply pre-built vertical profiles for Clinics, Restaurants, E-Commerce brands, and Service Agencies.

### 3.2 Framework & Sandboxed Runtime (`frappe-framework`)
- **FR-FRAME-01**: Must enforce document state machine transitions (`Draft` $\to$ `Submitted` $\to$ `Cancelled`).
- **FR-FRAME-02**: Must isolate user-defined plugins in a WASI 0.2 Wasmtime sandbox with deterministic fuel metering ($1,000,000$ operations limit).
- **FR-FRAME-03**: Must provide an RBAC-gated `ErpToolDispatcher` enabling AI agents to execute strongly typed ERP functions securely.

### 3.3 Persistence, CRDTs & Cryptography (`frappe-storage`)
- **FR-STOR-01**: Must embed SurrealDB in-memory or on disk with zero-leak connection pooling and isolated namespaces (`tenant_{id}`).
- **FR-STOR-02**: Must provide state-based PN-Counters, Vector Clocks, and Last-Write-Wins Document join-semilattices.
- **FR-STOR-03**: Must maintain a durable `OfflineOutboxManager` queuing edge mutations during network partitions.
- **FR-STOR-04**: Must build tamper-evident SHA-256 Merkle trees across transaction audit logs.
- **FR-STOR-05**: Must enforce field-level AEAD envelope encryption pairing Master KEKs with per-tenant DEKs.

### 3.4 Networking, Multi-Tenancy & ACME (`frappe-net`)
- **FR-NET-01**: Must resolve tenant identity from headers (`X-Tenant-Id`) or subdomain routing into a strongly typed `TenantContext`.
- **FR-NET-02**: Must provide an in-process automated ACME gateway negotiating and caching Let's Encrypt TLS certificates.
- **FR-NET-03**: Must support `--micro` runtime flags bounding memory caches and background worker queues.
- **FR-NET-04**: Must stream real-time document mutation events over WebSocket connections.

### 3.5 Financial & Accounting Systems (`erp-accounting`)
- **FR-ACC-01**: Must enforce balanced multi-currency double-entry journal postings with zero floating-point drift.
- **FR-ACC-02**: Must detect statistical transaction anomalies using Benford's Law Chi-Square goodness-of-fit (`BenfordGuard`).
- **FR-ACC-03**: Must generate non-interactive Zero-Knowledge Balance Sheet Proofs (`ZkProofEngine`) attesting to solvent books in zero knowledge.

### 3.6 Inventory, Logistics & Manufacturing
- **`erp-inventory`**: Real-time Stock Ledger Entries (SLE), SIMD-aligned FIFO cost valuation queues (`consume_fifo`), and batch/serial expiration validation.
- **`erp-manufacturing`**: Multi-level recursive BOM explosion, Mixed-Integer Linear Programming (MILP) job shop scheduling, and Statistical Process Control (SPC) with Nelson rules.
- **`erp-wms`**: 3D volumetric slotting, TSP pick path optimization, GS1 SSCC-18 handling units, and VDA 5050 AMR robot dispatch.
- **`erp-trade`**: Cascading pricing rule matrices, metered billing models, landed cost distribution, automated 3-way invoice matching, and WooCommerce streaming ingestion.

### 3.7 Projects, Assets & Specialized Industry Modules
- **`erp-ppm`**: Dual-engine CPM/CCPM project scheduling, ANSI/EIA-748 EVMS metrics (CPI, SPI, EAC, TCPI), and 100k-run Monte Carlo risk simulator.
- **`erp-asset`**: Linear Referencing Systems (LRS), Weibull $(\beta, \eta)$ Remaining Useful Life (RUL) predictive degradation, and tamper-evident PTW / LOTO safety interlocks.
- **`erp-software`**: ASC 606 5-step revenue recognition, graduated SaaS subscription tiers, and SLA penalty credit ledgers.
- **`erp-cms`**: Declarative `PageBlock` AST, pre-allocated SSR HTML renderer ($<10\,\text{ms}$ TTFB), atomic e-commerce checkout, LuxeGen 3D WebGL configurator (Three.js PBR + GSAP kinetic typography), and 12 universal work-type templates.
- **`erp-hr`**: Attendance logs, biometric tracking, multi-component salary structures, and automated payroll batches.
- **`erp-crm`**: Multi-stage lead acquisition funnels, deal conversion pipelines, and weighted opportunity scoring.
- **`erp-support`**: Ticket SLA countdown timers and priority escalation matrices.
- **`erp-lending`**: Reducing-balance and compound interest accrual with exact-decimal amortization schedules.
- **`erp-learning`**: Course curriculums, lesson tracking, student enrollment, and automated quiz scoring.

---

## 4. Quality & Compliance Standards

- **Rust 2024 / 2021 Edition Compliance**: Pure memory-safe Rust with zero `.unwrap()` in production paths.
- **Zero-Warning Policy**: All code must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
- **Formatting Standard**: 100% compliance with `cargo fmt --check`.
- **Test Coverage**: 100% test pass rate across unit tests and the 10-Epoch verification suite.
