# ledger

## Classs

- [AccountingError](AccountingError.md) — Accounting domain errors.
- [GlEntry](GlEntry.md) — Immutable General Ledger Entry row.
- [JournalEntry](JournalEntry.md) — A complete journal entry transaction document.
- [JournalEntryLine](JournalEntryLine.md) — An individual debit/credit line item in a Journal Entry.
- [LedgerPostingEngine](LedgerPostingEngine.md) — Atomic Ledger Posting Pipeline.
- [PeriodClosingLog](PeriodClosingLog.md) — Period closing record sealing fiscal transactions.
- [StatementGenerator](StatementGenerator.md) — Real-time Financial Statement Generator (Milestone 2.4).
- [TrialBalanceRow](TrialBalanceRow.md) — Trial Balance row summary.

## Functions

- [balance_exchange_variance](balance_exchange_variance.md) — Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.
- [balance_exchange_variance](balance_exchange_variance_1.md) — Automatically generates balancing Exchange Gain/Loss line when settlement exchange rates differ.
- [calculate_net_profit](calculate_net_profit.md) — Computes Net Profit ($\sum \text{Income} - \sum \text{Expense}$).
- [calculate_net_profit](calculate_net_profit_1.md) — Computes Net Profit ($\sum \text{Income} - \sum \text{Expense}$).
- [generate_trial_balance](generate_trial_balance.md) — Generates real-time Trial Balance.
- [generate_trial_balance](generate_trial_balance_1.md) — Generates real-time Trial Balance.
- [new](new.md) — Creates a new posting engine instance.
- [new](new_1.md) — Creates a new posting engine instance.
- [post_journal_entry](post_journal_entry.md) — Posts a journal entry to the immutable general ledger.
- [post_journal_entry](post_journal_entry_1.md) — Posts a journal entry to the immutable general ledger.
- [validate_balance](validate_balance.md) — Validates the zero-loss balancing equation $\sum \text{Debit} - \sum \text{Credit} = 0$.
- [validate_balance](validate_balance_1.md) — Validates the zero-loss balancing equation $\sum \text{Debit} - \sum \text{Credit} = 0$.
- [verify_balance_sheet](verify_balance_sheet.md) — Verifies the Fundamental Accounting Equation: $\text{Assets} = \text{Liabilities} + \text{Equity} + \text{Net Profit}$.
- [verify_balance_sheet](verify_balance_sheet_1.md) — Verifies the Fundamental Accounting Equation: $\text{Assets} = \text{Liabilities} + \text{Equity} + \text{Net Profit}$.
