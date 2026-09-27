# erp-manufacturing

Multilevel Bill of Materials (BOM) explosion, Work Orders, Workstations, and Material Requirement Planning (MRP).

---

## 📦 Overview

`erp-manufacturing` powers production planning, multi-tier assembly trees, scrap rate factors, and inventory requirement forecasts.

### Key Capabilities

- **BOM Explosion (`bom.rs`)**: Recursive BOM expansion with multi-level cost rollup and cycle detection.
- **MRP Engine (`mrp.rs`)**: Calculates net raw material demand against minimum order quantities and safety stock.
