# erp-trade

Pricing rule engines, item taxes, landed cost allocations, and purchasing/sales workflows.

---

## 📦 Overview

`erp-trade` handles commercial transactions, pricing rules, tax calculations, and landed cost distribution for ERPNext Rust.

### Key Capabilities

- **Pricing Rules (`pricing.rs`)**: Priority-based discount matrices, minimum quantity tiers, and promotional campaigns.
- **Taxes & Charges (`taxes.rs`)**: Multi-row tax breakdown, inclusive/exclusive taxes, and withholding tax formulas.
- **Landed Cost Allocation (`landed_cost.rs`)**: Allocates freight, customs, and insurance expenses proportionally by item value or quantity.
