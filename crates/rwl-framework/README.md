# ⚙️ rwl-framework

> **Document Lifecycle State Machines, Submittable Document Workflows, WASI Sandboxes & Rhai Scripting for `rust-web-land` (`rwl`)**

`rwl-framework` implements the core document lifecycle transitions, multi-stage approval workflows, and dual-engine sandboxing environment for `rust-web-land` (`rwl`).

---

## ⚡ Core Capabilities

- **Document Controller & Lifecycle Engine (`controller.rs`, `lifecycle.rs`)**:
  - Enforces deterministic document lifecycle hooks (`validate` → `before_save` → `on_update` → `before_submit` → `on_submit` → `before_cancel` → `on_cancel`).
  - Native Submittable (`docstatus: 0, 1, 2`) and amendment cycle handling.
- **Wasmtime WASI 0.2 Sandbox (`wasmtime_sandbox.rs`)**:
  - Fault-isolated WebAssembly component execution for enterprise extension plugins.
- **Embedded Rhai Scripting Engine**:
  - High-velocity embedded scripting for client validation logic, computed formula fields, and custom triggers.
- **Automated Workflow Approval Engine (`workflow_approval.rs`, `workflow_nodes.rs`)**:
  - Multi-tier visual approval graph execution with role gate checks and conditional branch routing.
- **1-Click Blueprints (`blueprints/`)**:
  - Modular scaffolding for ERP, CRM, CMS, LMS, and SaaS multi-tenant portals.

---

## 🛠 Usage Example

```rust
use rwl_framework::{ControllerRegistry, DocumentRecord};

let registry = ControllerRegistry::with_standard_submittables();
assert!(registry.is_submittable("Sales Invoice"));
```

---

Part of the **[rust-web-land](file:///home/jrad/RustroverProjects/ERPNext_workspace/README.md)** platform.
