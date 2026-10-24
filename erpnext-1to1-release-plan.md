# `rust-web-land` (`rwl`) — Enterprise Release Plan & Milestone Verification

## Goal
Deliver a stable, production-ready, pure-Rust universal platform with comprehensive business operational capabilities for real-world businesses across accounting, inventory, supply chain, manufacturing, commerce, and human resources, deployable on a 1GB VPS in 1 click.

## Tasks
- [x] Task 1: Fix SurrealQL readiness probe syntax (`RETURN 1;` in `rwl-net`) → Verify: `curl http://127.0.0.1:8080/healthz/ready` returns `{"status":"ready","db":"connected"}`
- [x] Task 2: Align multi-persona shell titles (`worker`, `admin`) in `desk-app` → Verify: Playwright `Multi-Persona SSR Desktop Shells` passes
- [x] Task 3: Unify commercial printable invoice title and doc-number formatting in `erp-accounting` → Verify: `cargo test -p rwl-net --test erpnext_v16_replacement_integration_tests` passes
- [x] Task 4: Synchronize enterprise template suites catalog (10 suites) and SEO titles in `erp-cms` → Verify: `npx playwright test --project=chromium` passes all 21 E2E tests
- [x] Task 5: Eliminate Clippy warnings and enforce zero-warning strict gate across workspace → Verify: `cargo clippy --workspace --all-targets -- -D warnings` exits with code 0
- [x] Task 6: Validate DocType schema completeness against standard master registry (44 canonical DocTypes) → Verify: `cargo test -p rwl-meta --test schema_compiler_tests` passes
- [x] Task 7: Execute double-entry accounting reconciliation and multi-currency ledger stress test → Verify: `cargo test -p erp-accounting` passes with zero decimal drift
- [x] Task 8: Benchmark multi-tenant HTTP/1.1, HTTP/2 and HTTP/3 transaction throughput with `rbench` → Verify: `cargo test -p rbench` passes all scaffolding, migration, and site provisioning tests
- [x] Task 9: Verify edge offline mutation buffer convergence with vector clocks and commutative counters → Verify: `cargo test -p desk-app` passes
- [x] Task 10: Run full cross-browser E2E verification suite with Playwright → Verify: `npx playwright test --project=chromium` passes all 21 test suites
- [x] Task 11: Mainstream 1:1 GUI Parity (AwesomeBar Ctrl+K autocomplete popover, collapsible sidebar, theme switcher, status filter pills, bulk action batch operations, sortable columns, CSV export, linked connections dashboard, child table editable grid with real-time arithmetic, and keyboard shortcuts) → Verify: `npx playwright test --project=chromium` passes all 24 E2E tests
- [x] Task 12: Production Hardening Gate: submitted document immutability, constant-time webhook signatures, wasmtime JIT module cache, secondary ledger indexes, QUIC flow control, bounded in-memory FIFO caches → Verify: `cargo test -p rbench --test security_hardening_tests` passes

## Done When
- [x] All 24 Playwright GUI/UX & API E2E tests pass cleanly (100% pass rate).
- [x] All workspace unit and integration test suites pass with 0 failures across all 32 crates.
- [x] Workspace compiles with 0 compiler warnings and passes `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 100% compliance with `cargo fmt --check`.
- [x] VPS memory envelope strictly capped at $\le 1\,\text{GB}$ RAM with embedded SurrealKV storage.

## Notes
- Persistence is handled by embedded SurrealDB 3.3.0 (SurrealKV) with zero IPC overhead and zero external database daemons.
- All business transactions enforce exact decimal arithmetic (`rust_decimal`), eliminating floating-point rounding errors.
