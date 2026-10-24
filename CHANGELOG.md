# Changelog

All notable changes to the `rust-web-land` (`rwl`) pure-Rust universal platform are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-10-18

### Added
- **Production OTT Video Streaming Subsystem (`erp-cms`)**:
  - HLS v6 / CMAF engine with multi-variant video ladders, multi-channel audio tracks, and WebVTT subtitles.
  - Server-Side Ad Insertion (SSAI) with SCTE-35 Cue-In / Cue-Out manifest stitching and RFC-compliant tracking beacons.
  - FAST linear channel engine with 24/7 EPG scheduling and live sliding-window playlists.
  - Stremio Addon Protocol v1 (`manifest.json`, catalog queries, meta detail, video streams, subtitle tracks).
  - Video X-Ray metadata system tracking actors, scene trivia, music tracks, and WebVTT metadata tracks at current playhead.
  - Resilient low-bandwidth streaming with QUIC / HTTP/3 optimization headers and user prefetch cache registry.
- **Visual Block Canvas AST & Emdash Editorial Studio (`erp-cms`)**:
  - Declarative reversible Block Canvas AST with slot containers, responsive breakpoint cascade, and SurrealQL two-way DocType binding.
  - High-performance compiled SSR engine rendering pre-allocated HTML in $<10\text{ms}$ TTFB.
  - Emdash editorial studio: multi-channel editorial workflows, revision diffing, focal-point cropping, and responsive WebP variant generation.
- **10 Enterprise Template Suites & 10 Aesthetic Archetypes (100 Permutations)**:
  - 10 full vertical template suites: `svod-streaming`, `lms-academy`, `digital-goods`, `b2b-industrial`, `b2c-retail`, `trading-exchange`, `real-estate-luxury`, `healthcare-clinic`, `restaurant-culinary`, `news-editorial`.
  - 10 design archetypes: `awwwards`, `cyberpunk`, `vaporwave`, `retrowave`, `neonwave`, `tasteful`, `brutalist`, `editorial`, `luxury`, `monochrome`.
  - Live preview switcher dock and runtime `<1ms` variant re-rendering via `?variant={name}`.
- **Master Audio Streaming CMS (Spotify, SoundCloud, Audioserve, Owncast & Navidrome)**:
  - Lossless audio ingestion and FLAC compression pipeline (levels 0..8) with 4 audiophile quality tiers and VorbisComment preservation.
  - Custom 10-band ISO equalizer with biquad IIR filter generation, automatic headroom preamp attenuation, and 10 presets.
  - Line-level and syllable/word karaoke LRC parser with active playhead resolution.
  - Music Intelligence Engine: technical liner notes (ISRC, BPM, Camelot wheel, contributors), Camelot harmonic key distance scoring, and transcript TF-IDF semantic recommendation engine.
  - EBU R128 loudness normalization (-14 LUFS, -1 dBFS true-peak limiter clamp) and gapless crossfading (`EqualPower`, `Linear`, `Logarithmic`).
  - SoundCloud-grade 100-point & 1800-point scrub bar waveform generators, mirror SVG vector rendering, and temporal comment clustering histogram.
  - Procedural sleep soundscapes (white, pink $1/f$, brown $1/f^2$), binaural beats synthesizer, multi-stem soundboard mixer, and exponential volume decay timer.
  - Owncast-grade live radio broadcasting with LL-HLS audio (`#EXT-X-TARGETDURATION:2`), real-time listener heartbeats, and live chat rooms.
  - Audioserve-grade audiobook chapter markers (M4B, ID3v2 CHAP) and Last-Write-Wins (LWW) cross-device playback position synchronization.
  - Subsonic / OpenSubsonic API v1.16+ protocol controller (`ping.view`, `getLicense.view`, `getSong.view`, `stream.view`, `getMusicFolders.view`, `scrobble.view`, `getNowPlaying.view`, `search3.view`).
  - Bare-metal `#![no_std]` wire playback telemetry packet (`AudioPlaybackPacket`) with CRC16-CCITT checksum for Cortex-M microcontrollers.

---

## [0.3.0] - 2026-10-12

### Added
- **Production Printable Commercial Document Engine (`print_format.rs`)**:
  - Implemented enterprise-grade printable commercial invoice, receipt, and sales order rendering engine in `erp-accounting`.
  - Full `@media print` CSS stylesheet with page margin constraints, vector SVG branding, line items table, multi-rate VAT breakdown, bank payment instructions (IBAN, BIC/SWIFT, routing), and authorized signatory blocks.
  - Mounted HTTP endpoints in `rwl-net`: `GET /api/v2/document/{doctype}/{name}/print` and `POST /api/v2/print/document`.
- **Pluggable Payment Gateway & Automated GL Reconciliation (`payment_gateway.rs`)**:
  - Built pluggable transaction processing abstraction (`PaymentGateway`, `PaymentIntent`, `PaymentStatus`: Authorized, Captured, Refunded, Failed) in `erp-accounting`.
  - Implemented `PaymentReconciliationEngine` automatically generating double-entry balanced Journal Entries and clearing Accounts Receivable against bank/clearing ledgers on capture.
  - Mounted HTTP endpoints in `rwl-net`: `POST /api/v2/payments/charge` and `POST /api/v2/payments/webhook`.
- **Unified Enterprise Desk Shell, List View & Form View (`desk-components` & `desk-app`)**:
  - Full SSR hypermedia administration desktop with real-time OmniBar filtering (`Ctrl+K`).
  - Dynamic List View (`/desk/list/{doctype}`) with search query filter, record counts, and SurrealDB pagination.
  - Responsive Form View (`/desk/form/{doctype}/{id}`) supporting field editing, Draft save, Submit lifecycle lock (`docstatus` 1), Cancellation (`docstatus` 2), and deletion.
  - Role-adaptive multi-persona views: `/desk` (Enterprise), `/portal` (Customer), `/worker` (Warehouse Kiosk), `/factory` (MES Operator), `/approvals` (Executive Triage), `/admin` (System Ops Cockpit).
- **GUI-UX Craft Polishing & Anti-Slop Cleansing (`/gui-ux-tester`)**:
  - Replaced all emoji glyphs with authentic inline SVG vector icons across desk shells, kiosk views, and print headers.
  - Added Emil Kowalski physics springs (`cubic-bezier(0.16, 1, 0.3, 1)`), button active press micro-interactions (`transform: scale(0.98)`), and Webkit scrollbar restyling.

### Removed (Ponytail Bloat Pruning)
- Removed speculative ZK proof engine (`zk_proof.rs`) and dummy video HUD (`video_hud.rs`).
- Removed CMS subtitle and transcoder stubs (`subtitles.rs`, `transcoder.rs`).
- Removed dead dependencies and streamlined workspace compilation footprint.

### Fixed
- Fixed Decimal serialization invariant in API v2 routes: exact string representations preserved for currency outputs while input deserializer flexibly parses JSON numbers and strings.
- Resolved SurrealDB payload resolution in form view JavaScript ensuring `json.doc` and `json.data` properly update document status and action triggers.

---

## [0.2.0] - 2026-09-28

### Added

- **Epoch I: Substrate Micro-Topology & Automated ACME Gateway**
  - Added `--micro` topology profile in `rwl-net` (8MB write buffer, 16MB read cache, bounded 1024-entry queues) for sub-64MB RSS operations.
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
  - HTTP endpoints in `rwl-net`: `GET /` & `GET /storefront`, `GET /api/v1/storefront/products`, and `POST /api/v1/storefront/checkout`.

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
