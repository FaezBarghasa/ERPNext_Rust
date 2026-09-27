# erp-inventory

Real-time stock ledger, warehouse hierarchy, FIFO queue valuation, and batch/serial number management.

---

## 📦 Overview

`erp-inventory` manages stock balances, valuation rates, and bin transfers across multi-tier warehouses.

### Key Capabilities

- **FIFO Queue Valuation (`fifo.rs`)**: Exact purchase queue matching for accurate cost of goods sold (COGS).
- **Warehouse Management (`warehouse.rs`)**: Hierarchical warehouse trees with balance aggregation.
- **Batch & Serial Tracking (`batches.rs`)**: Lot expiration enforcement and unique serial unit tracking.
