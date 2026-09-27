use serde::{Deserialize, Serialize};

/// Root account classification according to standard GAAP/IFRS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootType {
    /// Economic resources owned (e.g. Cash, Bank, Receivables, Inventory, Fixed Assets).
    Asset,
    /// Financial debts or obligations (e.g. Payables, Loans, Taxes).
    Liability,
    /// Net worth / Owner's residual interest (e.g. Share Capital, Retained Earnings).
    Equity,
    /// Gross revenue generated (e.g. Sales, Service Income, Interest Income).
    Income,
    /// Operational and administrative costs (e.g. COGS, Salary, Rent, Depreciation).
    Expense,
}

/// Chart of Accounts node record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// Unique account name / path (e.g. "1110 - Bank Account - ACME").
    pub name: String,
    /// Standard account number code.
    pub account_number: Option<String>,
    /// Parent group account name in tree.
    pub parent_account: Option<String>,
    /// Is this account a non-posting group folder?
    pub is_group: bool,
    /// Root GAAP classification.
    pub root_type: RootType,
    /// Currency code (e.g. "USD", "EUR", "IRR").
    pub account_currency: String,
}
