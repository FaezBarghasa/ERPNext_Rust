# erp-accounting

Double-entry general ledger, multi-currency conversion, Chart of Accounts, Asset depreciation, and receivables/payables tracking.

---

## 📦 Overview

`erp-accounting` guarantees mathematical and financial consistency through strictly balanced, immutable General Ledger entries.

### Key Capabilities

- **Double-Entry Ledger (`ledger.rs`, `decimal_ledger.rs`)**: Enforces zero-sum imbalance validation (`Σ Debit == Σ Credit`) with exact decimal precision.
- **Chart of Accounts (`coa.rs`)**: Tree-structured Chart of Accounts (Assets, Liabilities, Equity, Income, Expenses).
- **Fixed Asset Depreciation (`assets.rs`)**: Straight-line and reducing-balance depreciation computation schedules.
- **Receivables & Payables (`receivables.rs`)**: Aging buckets and payment reconciliation logic.
