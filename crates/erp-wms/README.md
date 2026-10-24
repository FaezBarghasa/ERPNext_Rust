# 🏭 erp-wms (`rust-web-land`)

> **Warehouse Management System (WMS) Engine for `rust-web-land` (`rwl`)**.  
> High-performance multi-bin warehouse operations, barcode directed putaway, picking paths, batch/serial routing, and inventory zoning in pure Rust.

---

## 📦 Key Capabilities

- **Zone & Bin Topology**: Multi-dimensional warehouse hierarchy (Warehouse -> Zone -> Aisle -> Rack -> Shelf -> Bin).
- **Directed Putaway & Picking**: Path-optimized algorithms minimizing transit time across industrial layouts.
- **Batch & Serial Traceability**: Strict lineage enforcement from receiving to dispatch with zero heap allocations in hot loops.
- **Stock Ledger Synchronization**: Native integration with [`rwl-storage`](../rwl-storage) and [`erp-inventory`](../erp-inventory).

---

## 🛠️ Usage Example

```rust
use erp_wms::warehouse::{WarehouseLayout, Zone, Bin};

let mut layout = WarehouseLayout::new("WH-CENTRAL");
let zone_a = Zone::new("ZONE-A", "High Turnover");
layout.add_zone(zone_a);
```
