# 🏛️ erp-asset

> **Fixed Asset Management, Capitalization, Depreciation & Industrial Safety for `rust-web-land` (`rwl`)**

`erp-asset` delivers enterprise-grade fixed asset lifecycle management, complex depreciation schedules, Lockout-Tagout (LOTO) safety permit workflows, MRO Poisson spares optimization, and Linear Referencing System (LRS) route tracking.

---

## ⚡ Core Capabilities

- **Depreciation Engine (`depreciation.rs`)**:
  - Straight-Line, Double Declining Balance, Written Down Value, and Manual depreciation calculation.
  - Asset Capitalization & Work-in-Progress (CWIP) compounding to active asset conversion.
  - Automated journal entry allocation and asset depreciation schedule projections.
- **Safety & Work Permits (`safety.rs`)**:
  - Lockout-Tagout (LOTO) isolation point classification (Electrical, Pneumatic, Hydraulic, Mechanical).
  - Permit-to-Work issuance, sign-off validation, and multi-technician release locks.
- **Maintenance & MRO Optimization (`mro.rs`)**:
  - Poisson distribution spares optimizer for critical replacement components.
  - Preventative maintenance periodicity (Daily, Weekly, Monthly, Operating Hours, Metric Units).
- **Linear Referencing System (`lrs.rs`)**:
  - Distance marker interpolation for continuous linear infrastructure (pipelines, rail, roads, conveyor systems).
- **Aerospace & Reliability Analytics (`reliability.rs`)**:
  - Optional `aerospace-lrs` feature with Weibull distribution failure modeling and FMECA risk scoring.

---

## 📦 Workspace Architecture

```
crates/erp-asset/
├── Cargo.toml
└── src/
    ├── depreciation.rs     # Asset ledger, schedules & capitalization
    ├── lib.rs              # Public exports & trait implementations
    ├── lrs.rs              # Linear referencing coordinate engine
    ├── mro.rs              # MRO Poisson inventory optimization
    ├── reliability.rs      # Weibull & FMECA reliability analytics
    └── safety.rs           # LOTO tags & Permit-to-Work state machines
```

---

## 🚀 Quick Usage

```rust
use erp_asset::{Asset, AssetParams, DepreciationMethod};
use rust_decimal_macros::dec;

let params = AssetParams {
    gross_purchase_amount: dec!(100000.00),
    expected_value_after_useful_life: dec!(10000.00),
    total_number_of_depreciations: 60,
    depreciation_method: DepreciationMethod::StraightLine,
};

let mut asset = Asset::new("AST-0001", "CNC Milling Machine", params);
asset.calculate_schedule();
assert_eq!(asset.schedule.len(), 60);
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** ecosystem.
