# `desk-components` — Reactive UI Component Library & Visual Workspace Views

`desk-components` provides the shared UI building blocks, dynamic form synthesis engines, virtualized grids, and interactive operational visualizations for the ERPNext Rust client layer across Desktop, Web, and Mobile.

---

## 🧩 Component Architecture

| Module | Implementation | Core Capabilities |
| :--- | :--- | :--- |
| **Desk Shell** | `desk_shell.rs` | Universal command center, instant OmniBar filtering (`Ctrl+K`), workspace shortcuts, multi-persona switcher, real-time alert badges. |
| **Dynamic Form Engine** | `form_engine/`, `forms.rs` | Compiles `DocTypeSchema` into interactive inputs with strict decimal validation, dependency cascading, and submit/cancel lifecycle actions. |
| **Virtualized Grid** | `grid.rs` | High-performance virtualized data table rendering up to $1{,}000{,}000$ rows at sustained 60 FPS with sortable headers, search, and pagination. |
| **CPM Gantt Schedule** | `gantt.rs` | Interactive project schedule timeline rendering critical path nodes, early/late slack floats, and task dependencies. |
| **SPC Control Chart** | `spc_view.rs` | Real-time Nelson-rule statistical process control chart detecting 3-sigma excursions, run trends, and mean drift. |
| **WMS 3D Slotting** | `wms_view.rs` | Interactive warehouse rack visualizer displaying pallet slotting, bin capacity heatmaps, and VDA 5050 AMR robot vectors. |
| **CPQ Configurator** | `cpq_view.rs` | Configure-Price-Quote 3D dimensional interactive parameter viewer. |
| **Report Query Builder** | `report_query.rs` | Interactive visual query builder generating parameterized SurrealQL aggregation pipelines. |
| **Design Primitives** | `primitives.rs`, `views.rs` | Atomic buttons, status chips (`Draft`, `Submitted`, `Cancelled`), modals, slide-over drawers, and toast stacks. |

