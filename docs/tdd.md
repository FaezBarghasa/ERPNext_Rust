# Test-Driven Development (TDD) Guide

## 1. TDD Philosophy & Engineering Invariants

The `rustnext` workspace adheres to a strict Test-Driven Development methodology:

1. **Red**: Write a failing unit or integration test defining domain invariants (e.g., zero accounting drift, WASI fuel exhaustion, FIFO layer consumption, ZK proof validity).
2. **Green**: Implement the minimal, correct, memory-safe Rust logic satisfying the specification.
3. **Refactor**: Eliminate allocation bottlenecks, remove dead code, enforce strict type safety, and verify zero compiler or linter warnings.

---

## 2. Test Architecture

### 2.1 Unit Tests
- Co-located in `src/` modules within `#[cfg(test)] mod tests { ... }`.
- Validate pure algorithmic logic:
  - Exact FIFO queue consumption and valuation.
  - Multi-component payroll and tax deductions.
  - Chi-Square Benford's Law anomaly detection.
  - CPM schedule pass calculations (Early Start/Finish, Late Start/Finish, Floats).
  - GS1 SSCC-18 check digit verification.
  - Nelson rules for SPC control charts.
  - CRDT vector clock merging and join-semilattices.
- Fast, hermetic, and deterministic with zero I/O side effects.

### 2.2 Integration Test Suites
- Reside in `tests/` directories within crates:
  - [charter_all_epochs_test.rs](file:///home/jrad/RustroverProjects/ERPNext_workspace/crates/frappe-net/tests/charter_all_epochs_test.rs): Comprehensive end-to-end integration test exercising all 10 Epochs.
  - [tenant_isolation_tests.rs](file:///home/jrad/RustroverProjects/ERPNext_workspace/crates/frappe-net/tests/tenant_isolation_tests.rs): Validates zero cross-tenant contamination in database namespaces.
  - [schema_compiler_tests.rs](file:///home/jrad/RustroverProjects/ERPNext_workspace/crates/frappe-meta/tests/schema_compiler_tests.rs): Validates dynamic SurrealQL schema compilation and backward-compatible migrations.

---

## 3. Crate-by-Crate Verification Targets

| Crate | Primary Test Targets | Invariants Verified |
| :--- | :--- | :--- |
| **`frappe-meta`** | DynamicDoc, Schema Compiler, AI Synthesizer, Profiles | Stack allocation limit (SmallVec), valid SurrealQL DDL, profile existence |
| **`frappe-framework`** | Lifecycle Engine, WASI Sandbox, AI Tools | Strict state transitions, fuel exhaustion trapping, RBAC tool permissions |
| **`frappe-storage`** | CRDTs, Outbox Queue, Envelope Encryption, Merkle | Monotonic clock growth, LWW convergence, AEAD roundtrip, Merkle root hash |
| **`frappe-net`** | ACME Gateway, Scoped Sessions, Micro-Topology | Domain routing, tenant context extraction, connection pool isolation |
| **`erp-accounting`** | General Ledger, Benford Guard, ZK Proofs | $\sum \text{Debit} - \sum \text{Credit} = 0$, Chi-Square p-value, ZK proof $<15\,\text{ms}$ |
| **`erp-inventory`** | FIFO Cost Valuation, Batches & Serials | Positive quantities, FIFO depletion order, expiration date guards |
| **`erp-manufacturing`** | BOM Explosion, MILP Scheduler, SPC Engine | Acyclic BOMs, finite capacity constraint satisfaction, Nelson rule triggers |
| **`erp-ppm`** | EVMS Engine, CPM/CCPM Scheduler, Monte Carlo | CPI/SPI calculation, critical path identification, distribution confidence |
| **`erp-wms`** | 3D Slotting, TSP Picker, VDA 5050 AMR | Bin volume/weight capacity, route minimization, robot order dispatch |
| **`erp-asset`** | LRS Dynamic Segmentation, Weibull RUL, PTW/LOTO | Linear chain continuity, hazard rate math, cryptographic safety signatures |
| **`erp-software`** | ASC 606 SSP Allocation, SaaS Subscriptions, SLA | Revenue allocation invariant, tiered billing accuracy, SLA breach credits |
| **`erp-trade`** | Pricing Rules, 3-Way Match, WooCommerce Ingest | Tiered discount break matching, $\le 0.5\%$ matching tolerance, SKU mapping |
| **`erp-cms`** | Block Canvas, SSR Engine, Checkout Pipeline | $<10\,\text{ms}$ TTFB rendering, atomic multi-tier transaction posting |
| **`erp-hr`** | Attendance Logs, Salary Rules, Payroll Slips | Gross to net earnings reconciliations, statutory deduction brackets |
| **`erp-crm`** | Lead Intake, Opportunity Scoring, Deals | Lead progression state machine, weighted scoring convergence |
| **`erp-support`** | SLA Timers, Priority Matrices, Escalations | SLA breach countdown tracking, deterministic escalation rules |
| **`erp-lending`** | Loan Disbursement, Amortization Tables | Reducing balance interest accruals, exact EMI balance repayment |
| **`erp-learning`** | Course Syllabi, Enrollments, Quizzes | Prerequisite progression checks, quiz score grading thresholds |

---

## 4. Running the Test Suite

```bash
# Run all unit and integration tests across the entire workspace
cargo test --workspace

# Run the 10-Epoch master verification integration suite
cargo test --package frappe-net --test charter_all_epochs_test

# Run tests with full backtrace for deep diagnostics
RUST_BACKTRACE=1 cargo test --workspace

# Run synthetic throughput and latency benchmarks
cargo bench -p rbench
```

---

## 5. Continuous Quality Gates

Every code change must satisfy all three quality gates:
1. **Formatting Gate**: `cargo fmt --check` (0 formatting diffs).
2. **Clippy Gate**: `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings).
3. **Test Gate**: `cargo test --workspace` (100% passing tests).
