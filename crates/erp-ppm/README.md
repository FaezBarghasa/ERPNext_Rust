# 📊 erp-ppm

> **Project & Portfolio Management, Earned Value Management (EVM), CPM Scheduling & AIA Construction Billing for `rust-web-land` (`rwl`)**

`erp-ppm` provides industrial-grade project controls, Critical Path Method (CPM) project scheduling, Earned Value Management (EVM) ANSI-748 metrics, construction AIA G702 billing, and Work Breakdown Structure (WBS) trees.

---

## ⚡ Core Capabilities

- **Critical Path Method (CPM) Scheduler (`scheduler.rs`)**:
  - Early start, early finish, late start, late finish forward/backward scheduling passes.
  - Total float, free float, and project buffer sizing.
  - Multi-relationship task dependency graph (Finish-to-Start, Start-to-Start, Finish-to-Finish, Start-to-Finish).
- **Earned Value Management (`evm.rs`)**:
  - ANSI-748 EVM calculations: Planned Value (PV), Earned Value (EV), Actual Cost (AC).
  - Performance Indices: Cost Performance Index (CPI), Schedule Performance Index (SPI), To-Complete Performance Index (TCPI).
  - Variances: Cost Variance (CV), Schedule Variance (SV), Estimate at Completion (EAC).
- **AIA G702 Construction Billing (`construction.rs`)**:
  - Application and Certificate for Payment (AIA Document G702/G703).
  - Schedule of Values (SOV) retainage calculations, change order approvals, and conditional/unconditional lien waivers.
- **Hierarchical Breakdown Structures (`wbs.rs`)**:
  - Work Breakdown Structure (WBS), Cost Breakdown Structure (CBS), Resource Breakdown Structure (RBS), Organization Breakdown Structure (OBS).
  - QuadMatrix structural reconciliation.
- **Timesheet Billing & Completion Engine (`timesheet_billing.rs`, `project_completion.rs`)**:
  - Percent complete methodologies (EVM physical progress, cost ratio, duration ratio, manual milestones).
  - Billable hours ledger integration and automated project invoice generation.

---

## 📦 Workspace Architecture

```
crates/erp-ppm/
├── Cargo.toml
└── src/
    ├── construction.rs         # AIA G702/G703 certificates & lien waivers
    ├── evm.rs                  # ANSI-748 Earned Value Management engine
    ├── lib.rs                  # Public exports & trait implementations
    ├── monte_carlo.rs          # Optional probabilistic schedule simulation
    ├── project.rs              # Project & Task domain entity state machines
    ├── project_completion.rs   # Percent complete aggregation engines
    ├── scheduler.rs            # Critical Path Method (CPM) scheduler
    ├── timesheet_billing.rs    # Project time tracking & invoice generator
    └── wbs.rs                  # WBS, CBS, RBS & OBS quad matrices
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** ecosystem.
