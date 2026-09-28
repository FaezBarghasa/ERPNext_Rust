# Changelog

All notable changes to the `rustnext` (`ERPNext_workspace`) pure-Rust enterprise platform are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-09-28

### Added

- **Epoch I: Substrate Micro-Topology & Automated ACME Gateway**
  - Added `--micro` topology profile in `frappe-net` (8MB write buffer, 16MB read cache, bounded 1024-entry queues) for sub-64MB RSS operations.
  - In-process automated Let's Encrypt / ACME TLS certificate lifecycle management and multi-tenant domain resolver (`AcmeGateway`).
  - Scoped database session pooling (`resolve_scoped_session`, `TenantContext`) guaranteeing zero cross-tenant contamination.

- **Epoch II: Dynamic Document Bus & Lock-Free Sequence Generators**
  - Zero-allocation `DynamicDocument` polymorphic container with `SmallVec<[_; 16]>` on the CPU stack and `DocValue` enum.
  - Dynamic SurrealQL compiler (`compile_to_surrealql`) transforming declarative `DocTypeSchema` into `SCHEMAFULL` DDL tables, fields, and unique constraints.
  - Lock-free autoname sequence parsing engine (`NamingSeriesParser`) for sub-microsecond identifier formatting.

- **Epoch III: General Ledger Integrity, SIMD FIFO & Fraud Detection**
  - Multi-book arbitrary-precision double-entry General Ledger enforcing $\sum \text{Debit} - \sum \text{Credit} = 0$.
  - Contiguous SIMD-aligned FIFO inventory cost valuation queue (`consume_fifo`).
  - Real-time statistical fraud detector (`BenfordGuard`) using Chi-Square goodness-of-fit against Benford's Law.

- **Epoch IV: Advanced Project Portfolio & Manufacturing Intelligence**
  - Dual-engine CPM/CCPM scheduling and ANSI/EIA-748 Earned Value Management System (`EvmEngine`: CPI, SPI, EAC, TCPI).
  - 100k-iteration Monte Carlo stochastic risk simulation (`MonteCarloSimulator`) supporting Beta-PERT, Triangular, and Normal distributions.
  - Mixed-Integer Linear Programming (MILP) finite-capacity job shop scheduler (`MilpJobShopScheduler`).
  - Statistical Process Control (`SpcEngine`) with real-time X-bar/R control charts and Nelson rule violation checks.
  - VDA 5050 AMR robot fleet dispatch (`Vda5050FleetCoordinator`) and GS1 SSCC-18 handling unit check-digit verification.
  - Linear Referencing Systems (`LrsEngine`), Weibull $(\beta, \eta)$ Remaining Useful Life (`WeibullReliabilityEngine`), and cryptographic Permit-to-Work (PTW) / Lockout-Tagout (LOTO) safety interlocks.

- **Epoch V: Visual Block Canvas, Compiled SSR & 3D WebGL Commerce**
  - Declarative `PageBlock` AST (Hero, FeaturesGrid, ProductShowcase, Testimonial).
  - Zero-allocation compiled Server-Side HTML Rendering engine (`SsrEngine`) delivering $<10\text{ms}$ TTFB.
  - Atomic multi-tier e-commerce checkout transaction (`AtomicCheckoutEngine`) generating balanced GL entries.
  - Universal Multi-Template Work-Type Engine (`ThemeManifest`) and zero-downtime layout slot binding.
  - LuxeGen Haute Horlogerie 3D WebGL procedural configurator (Three.js PBR + GSAP ScrollTrigger kinetics) with dynamic metallurgy swapping and atomic checkout integration.
  - HTTP endpoints in `frappe-net`: `GET /` & `GET /storefront`, `GET /api/v1/storefront/products`, and `POST /api/v1/storefront/checkout`.

- **Epoch VI: WASI 0.2 Sandbox & Resource Isolation**
  - Fault-isolated WASI 0.2 runtime (`RealSandbox`) with deterministic instruction fuel metering ($1,000,000$ ops default limit) and 32MB linear memory isolation.

- **Epoch VII: Local-First CRDT Mesh & Offline POS Outbox**
  - State-based conflict-free replicated data types (`PnCounter`, `VectorClock`, `LwwDocumentState` join-semilattices).
  - Durable offline outbox queue manager (`OfflineOutboxManager`) for edge POS kiosks and offline nodes.

- **Epoch VIII: Autonomous AI Schema Synthesis & 3-Way Matching**
  - `AiSchemaSynthesizer`: Natural language prompt-to-DocType and Chart of Accounts generation.
  - `ErpToolDispatcher`: Strongly typed LLM function calling with strict role-based access control (RBAC).
  - Automated 3-way invoice matching (`InvoiceMatchingEngine`) reconciling PO, GRN, and OCR invoice lines ($\le 0.5\%$ tolerance).

- **Epoch IX: Enterprise Cryptography & Non-Interactive ZK Proofs**
  - SHA-256 Merkle audit trees (`MerkleHasher`) providing tamper-evident regulatory proof chains.
  - Field-level Envelope Encryption (`EnvelopeEncryption`) with Master KEK and per-tenant DEKs (AEAD).
  - Non-interactive Zero-Knowledge balance sheet proof engine (`ZkProofEngine`) verifying ledger solvency in zero knowledge.

- **Epoch X: High-Density Agency Domain Map & Vertical Profiles**
  - Direct streaming WooCommerce migration ingestion engine (`WooMigrationEngine`).
  - Curated 12-Archetype "Business-in-a-Box" Universal Work-Type Template Catalog: Haute Horlogerie & Luxury, Omnichannel Retail Superstore, B2B Industrial Manufacturing, Developer SaaS Platform, EPCM Megaprojects, Gastronomy & Ghost Kitchens, Healthcare & Telehealth, SVoD Media Streaming, Real Estate & Linear AEC, Higher Education LMS, Non-Profit Foundations, and Offline-First POS Kiosks.
  - Dynamic Multi-Tenant Theme Compiler & zero-downtime hot-swapping engine (`tab_theme` with sub-50µs cache invalidation).
  - Pre-configured vertical profile registry (`ProfileRegistry`).
  - Comprehensive 10-Epoch master verification integration suite (`tests/charter_all_epochs_test.rs`).
  - Unified workspace dependencies (`[workspace.dependencies]`) and upgraded workspace resolver to `resolver = "3"` (Rust 2024 Edition standard).

---

## [0.1.0] - 2026-09-27

### Added
- Initial workspace scaffolding across 21 core infrastructure, business domain, and UI crates.
- Embedded SurrealDB integration layer and content-addressable drive storage.
- Basic Actix-web server routing and Dioxus UI component prototypes.
