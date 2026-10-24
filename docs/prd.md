# `rust-web-land` (`rwl`) — Product Requirements Document (PRD)

## 1. Executive Summary

`rust-web-land` (`rwl`) is a universal, no-code web application builder platform and planetary-scale enterprise operating substrate in pure Rust. It empowers individuals and teams with **zero technical knowledge** to visually design, customize, and deploy complete applications (CMS, LMS, SaaS, ERP, or All-in-One) with minimal effort, strict resource caps ($\le 1\,\text{GB}$ RAM budget on low-cost $3–$5/mo VPS or ARM single-board computers), and an automated zero-downtime CI/CD incremental upgrade pipeline.

---

## 2. Quantitative System Goals & SLAs

| Requirement Category | Metric / Constraint | Target Objective |
| :--- | :--- | :--- |
| **VPS Memory Budget** | Total RSS under peak workload | **Strictly $\le 1024\,\text{MB}$ RAM** ($<45\,\text{MB}$ idle, $<250\,\text{MB}$ active) |
| **Server-Side Rendering (SSR)** | Time to First Byte (TTFB) | $< 10\,\text{ms}$ on dynamic block canvas pages |
| **Binary Startup Time** | Cold Boot Latency | $< 35\,\text{ms}$ to ready state with embedded SurrealKV |
| **Site Provisioning Speed** | Blueprint Scaffolding & Deploy | $\le 2000\,\text{ms}$ for new multi-tenant instance |
| **WASI 0.2 Sandbox Trap** | Execution Time & Resource Ceiling | Traps loops in $\le 1.2\,\text{ms}$ with $32\,\text{MB}$ memory cap |
| **Accounting Balance Drift** | Double-Entry Invariant | Exactly $0.00\,\text{dec}$ ($\sum \text{Debit} - \sum \text{Credit} = 0$) |
| **ZK Balance Sheet Proof** | Verification Time | $< 15\,\text{ms}$ non-interactive verification |
| **3-Way Invoice Matching** | Price & Quantity Variance | Automated approval within $\pm 0.5\%$ variance |
| **Local-First CRDT** | Convergence Guarantee | Strongly eventual consistency with zero data loss |
| **3D WebGL Storefront Performance** | Sustained Frame Rate & Memory | $\ge 60\,\text{FPS}$ with $\le 18\,\text{MB}$ memory overhead |

---

## 3. Comprehensive Functional Requirements

### 3.1 Universal CLI & Pipeline Substrate (`rwl`, `tools/doctype-diff`)
- **FR-RWL-01**: Must provide `rwl init <name> --blueprint <cms|lms|saas|erp>` to scaffold turnkey projects pre-populated with standard DocTypes, visual page trees, and role matrices in $<100\,\text{ms}$.
- **FR-RWL-02**: Must provide `rwl studio --port <port>` to serve the native/browser visual drag-and-drop workspace at `/builder`.
- **FR-RWL-03**: Must provide `rwl export --target <vps|docker|systemd>` generating stripped production bundles pre-configured with `MemoryMax=1024M` and embedded SurrealKV auto-tuning.
- **FR-RWL-04**: Must provide `rwl deploy --host <host> --port <port> --token <token>` executing zero-downtime incremental schema diffs and hot asset swaps.
- **FR-RWL-05**: Must provide `rwl serve` to run the standalone production server with embedded SurrealKV database (zero external database daemons).
- **FR-RWL-06**: Must provide `rwl status` reporting runtime RSS, active tenant count, and storage health.

### 3.2 Visual Desktop Studio & Component Engine (`desk-app`, `desk-components`)
- **FR-DESK-01**: Must provide visual drag-and-drop page canvas supporting slot containers, responsive breakpoint cascades, and two-way DocType data bindings.
- **FR-DESK-02**: Must provide interactive DocType schema builder enabling custom field additions without writing SQL or Python.
- **FR-DESK-03**: Must provide visual logic and workflow node editor powered by sandboxed Rhai script nodes.
- **FR-DESK-04**: Must project 5 specialized role personas (`ClientCustomer`, `WarehouseWorker` with $\ge 56\,\text{px}$ touch targets, `ShopfloorMes` with dual-witness sign-offs, `ManagerApprover` with swipe deck, and `SystemAdmin` cockpit).
- **FR-DESK-05**: Must support disconnected edge transactions with causal vector clocks (`VectorClock`), local mutation buffers (`LocalMutationBuffer`), and deterministic join semilattices (`CloudSyncArbiter`).
- **FR-DESK-06**: Must provide polymorphic `MobileHardwareAbstractionLayer` routing biometrics, barcode scanning, thermal printing, and GPS across Android JNI and W3C Web APIs.

### 3.3 Metadata & Dynamic Schema Compilation (`rwl-meta`)
- **FR-META-01**: Must dynamically compile declarative `DocTypeSchema` structs into `SCHEMAFULL` SurrealQL DDL statements with index and relationship definitions.
- **FR-META-02**: Must provide zero-allocation `DynamicDocument` containers backed by `SmallVec<[_; 16]>` on the CPU stack.
- **FR-META-03**: Must support lock-free sequential naming series generators (e.g. `DRN-.YYYY.-.#####`).
- **FR-META-04**: Must provide an autonomous `AiSchemaSynthesizer` generating normalized DocTypes and Chart of Accounts from natural language prompts.
- **FR-META-05**: Must supply pre-built vertical profiles for Clinics, Restaurants, E-Commerce brands, and Service Agencies.

### 3.4 Framework & Sandboxed Runtime (`rwl-framework`)
- **FR-FRAME-01**: Must enforce document state machine transitions (`Draft` $\to$ `Submitted` $\to$ `Cancelled`).
- **FR-FRAME-02**: Must isolate user-defined plugins in a WASI 0.2 Wasmtime sandbox with deterministic fuel metering ($1,000,000$ operations limit) and in-memory SHA-256 JIT bytecode caching.
- **FR-FRAME-03**: Must provide an RBAC-gated `ErpToolDispatcher` enabling AI agents to execute strongly typed platform functions securely.

### 3.5 Persistence, CRDTs & Cryptography (`rwl-storage`)
- **FR-STOR-01**: Must embed SurrealDB in-memory or on disk with zero-leak connection pooling and isolated namespaces (`tenant_{id}`).
- **FR-STOR-02**: Must provide state-based PN-Counters, Vector Clocks, and Last-Write-Wins Document join-semilattices.
- **FR-STOR-03**: Must maintain a durable `OfflineOutboxManager` queuing edge mutations during network partitions.
- **FR-STOR-04**: Must build tamper-evident SHA-256 Merkle trees across transaction audit logs.
- **FR-STOR-05**: Must enforce field-level AEAD envelope encryption pairing Master KEKs with per-tenant DEKs.

### 3.6 Networking, Multi-Tenancy & ACME (`rwl-net`)
- **FR-NET-01**: Must resolve tenant identity from headers (`X-Tenant-Id`) or subdomain routing into a strongly typed `TenantContext`.
- **FR-NET-02**: Must provide an in-process automated ACME gateway negotiating and caching Let's Encrypt TLS certificates.
- **FR-NET-03**: Must stream real-time document mutation events over WebSocket connections.
- **FR-NET-04**: Must deliver native HTTP/3 (QUIC via Quinn and H3) alongside HTTP/1.1 and HTTP/2 with zero-downtime ALPN negotiation and stream window flow control bounds.
- **FR-NET-05**: Must enforce Dynamic RBAC with permission bitmasks, constant-time webhook verification, and `LoginGuard` brute-force rate-limiting.
- **FR-NET-06**: Must enforce submitted document immutability: direct updates to submitted records (`docstatus == 1`) rejected with HTTP 400.

### 3.7 Financial, Accounting & Commerce Domains
- **`erp-accounting`**: Balanced multi-currency double-entry journal postings with zero floating-point drift, Benford's Law anomaly detection, and constant-time webhook verification.
- **`erp-inventory`**: Real-time Stock Ledger Entries (SLE), SIMD-aligned FIFO cost queues (`consume_fifo`), and batch/serial expiration validation.
- **`erp-trade`**: Cascading pricing rule matrices, metered billing models, landed cost distribution, automated 3-way invoice matching, and WooCommerce streaming ingestion.
- **`erp-software`**: ASC 606 5-step revenue recognition, graduated SaaS subscription tiers, and SLA penalty credit ledgers.

### 3.8 Industrial, Operational & Media Systems
- **`erp-manufacturing`**: Multi-level recursive BOM explosion, Mixed-Integer Linear Programming (MILP) job shop scheduling, and Statistical Process Control (SPC) with Nelson rules.
- **`erp-wms`**: 3D volumetric slotting, TSP pick path optimization, GS1 SSCC-18 handling units, and VDA 5050 AMR robot dispatch.
- **`erp-ppm`**: Dual-engine CPM/CCPM project scheduling, ANSI/EIA-748 EVMS metrics, and 100k-run Monte Carlo risk simulator.
- **`erp-asset`**: Linear Referencing Systems (LRS), Weibull Remaining Useful Life (RUL) predictive degradation, and safety LOTO interlocks.
- **`erp-cms` & `erp-cms-core`**: Block Canvas AST, pre-allocated SSR HTML renderer ($<10\,\text{ms}$ TTFB), 10 vertical suites with 10 themes (100 permutations total), full OTT Video Delivery (HLS v6, SCTE-35 SSAI, FAST, Stremio v1), Master Audio Streaming CMS, and bare-metal `![no_std]` 63-byte wire telemetry framing.

---

## 4. Quality & Compliance Standards

- **Strict $\le 1\,\text{GB}$ VPS Envelope**: Deployable on low-cost $3–$5/mo VPS instances with zero swap thrashing.
- **Rust 2024 Edition Compliance**: Pure memory-safe Rust with zero `.unwrap()` in production paths.
- **Zero-Warning Policy**: All code must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
- **Formatting Standard**: 100% compliance with `cargo fmt --check`.
- **Test Coverage**: 100% test pass rate across all 32 workspace crates and regression test suites.

---

## 5. Related Engineering Documents

- 🚀 [**3-Minute Quickstart Guide (`docs/quickstart_3min_guide.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/quickstart_3min_guide.md)
- 🏛 [**System Architecture Specification (`docs/architecture.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/architecture.md)
- 📜 [**Master Architectural Evolution Charter (`docs/master_charter.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/master_charter.md)
- 🛠 [**Developer & Administrator Setup Guide (`docs/setup_guide.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/setup_guide.md)
- 👤 [**Universal Operator & User Guide (`docs/user_guide.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/user_guide.md)
- 🎨 [**Universal Templates & Theme Engine Guide (`docs/templates_guide.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/templates_guide.md)
- 🧪 [**Test-Driven Development (TDD) Specification (`docs/tdd.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/tdd.md)
- 🧪 [**End-to-End Testing & Verification Plan (`docs/e2e_testing_plan.md`)**](file:///home/jrad/RustroverProjects/ERPNext_workspace/docs/e2e_testing_plan.md)
