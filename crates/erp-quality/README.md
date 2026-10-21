# 🛡️ erp-quality

> **Enterprise Quality Control, ISO 9001 Non-Conformance (NCR) & CAPA Engine for `rust-web-land` (`rwl`)**

`erp-quality` delivers industrial-grade ISO 9001 quality assurance: parameterized inspection templates, statistical process control readings, Non-Conformance Reports (NCR) with multi-level severity disposition, and Corrective and Preventive Action (CAPA) tracking.

---

## ⚡ Core Capabilities

- **Quality Inspection Engine (`inspection.rs`)**:
  - Parameterized inspection criteria with min/max tolerance boundaries and acceptance criteria.
  - Multi-reading recording, statistical mean/median verification, and pass/fail automation.
  - Inspection templates reusable across Inward Goods, In-Process Manufacturing, and Final Pre-Shipment audits.
- **Non-Conformance Management (`ncr.rs`)**:
  - NCR generation with multi-tiered severity scoring (Minor, Major, Critical).
  - Material dispositions: Scrap, Rework, Return to Vendor, Concession/Use-As-Is.
  - Quarantine warehouse bin movements and automated inventory hold flags.
- **CAPA Lifecycle Engine (`capa.rs`)**:
  - Root Cause Analysis (5-Whys, Ishikawa fishbone) integration.
  - Preventive and corrective action items with assignment, due date gating, and effectiveness verification.

---

## 📦 Workspace Architecture

```
crates/erp-quality/
├── Cargo.toml
└── src/
    ├── capa.rs         # Corrective & Preventive Action (CAPA) engine
    ├── inspection.rs   # Quality inspections, templates & readings
    ├── lib.rs          # Public exports & error definitions
    └── ncr.rs          # Non-Conformance Reports (NCR) & dispositions
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** ecosystem.
