# Test-Driven Development (TDD) Guide

## 1. TDD Methodology & Quality Standards

All crates in the ERPNext Rust workspace must adhere to strict Test-Driven Development practices:

1. **Red**: Write a failing unit or integration test defining expected behavior.
2. **Green**: Implement the minimum correct logic necessary to satisfy the test.
3. **Refactor**: Clean up implementation, verify type boundaries, remove dead code, and ensure zero compiler warnings.

---

## 2. Test Structure & Guidelines

### Unit Tests
- Co-located within `src/` modules under `#[cfg(test)] mod tests { ... }`.
- Validate isolated logic, pure algorithmic calculations (e.g. FIFO queues, amortization math, tax breakdowns, SLA timers).
- Unit tests must be fast (<10ms per test) and deterministic with zero external network or filesystem I/O.

### Integration Tests
- Placed in `tests/` directory within individual crates.
- Test cross-module behavior, schema compilation, document lifecycles, and database interactions.

---

## 3. Crate-Specific Test Targets

### Core Crates
- **`frappe-meta`**: Test DocType schema parsing, RBAC permission evaluation for various roles, autoname format string expansion, and schema migration compatibility.
- **`frappe-framework`**: Test document validation pipelines, state transition invariants, and Rhai script sandboxing limits (timeout, memory allocation limits).
- **`frappe-storage`**: Test deduplication hashing, chunk storage integrity, and SurrealDB query mapping.
- **`frappe-net`**: Test tenant resolution middleware, WebSocket broadcast channels, and background queue task distribution.

### Financial & Business Crates
- **`erp-accounting`**: Test zero-sum balance assertion on journal entries, currency conversion precision, and asset depreciation calculations.
- **`erp-inventory`**: Test FIFO inventory depletion order, batch expiration checks, and warehouse stock transfers.
- **`erp-manufacturing`**: Test recursive BOM explosion cycles, scrap factor percentages, and material requirement aggregation.
- **`erp-trade`**: Test pricing rule priority matching, landed cost distribution math, and cascading tax calculations.
- **`erp-hr`**: Test attendance calculation against shifts, salary component formulas, and net pay computation.
- **`erp-lending`**: Test equal monthly installment (EMI) precision across variable loan durations and compound interest schedules.
- **`erp-cms`**: Test HMAC URL signing, expiration validation, subtitle parsing, and transcoding pipeline triggers.

---

## 4. Running Workspace Tests

```bash
# Run all unit and integration tests across the workspace
cargo test --workspace

# Run tests for a specific crate
cargo test -p erp-accounting
cargo test -p erp-inventory

# Run tests with backtrace for debugging
RUST_BACKTRACE=1 cargo test --workspace

# Run benchmarks
cargo bench -p rbench
```

---

## 5. Continuous Quality Gates

Every pull request or commit must pass:
- `cargo fmt --check` (Zero formatting errors)
- `cargo clippy --workspace -- -D warnings` (Zero lints or warnings)
- `cargo test --workspace` (100% test pass rate)
