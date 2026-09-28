use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Employee Cash / Travel Advance record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeAdvance {
    /// Advance record ID.
    pub name: String,
    /// Employee identifier.
    pub employee_id: String,
    /// Advance posting date.
    pub posting_date: NaiveDate,
    /// Currency code of the advance (e.g. "EUR", "USD", "IRR").
    pub currency: String,
    /// Advance amount in transaction currency.
    pub advance_amount: Decimal,
    /// Exchange rate to base company currency.
    pub exchange_rate: Decimal,
    /// Total advance amount in base company currency (`advance_amount * exchange_rate`).
    pub base_advance_amount: Decimal,
    /// Unallocated / remaining advance balance in base company currency.
    pub remaining_base_amount: Decimal,
}

/// Expense Claim line item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpenseClaimDetail {
    /// Expense type (e.g. "Airfare", "Hotel", "Meals").
    pub expense_type: String,
    /// Date expense was incurred.
    pub expense_date: NaiveDate,
    /// Currency code.
    pub currency: String,
    /// Claim amount in transaction currency.
    pub claim_amount: Decimal,
    /// Exchange rate to base company currency.
    pub exchange_rate: Decimal,
    /// Base claim amount (`claim_amount * exchange_rate`).
    pub base_claim_amount: Decimal,
}

/// Settled Expense Claim offsetting advances against expenses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettledExpenseClaim {
    /// Total expense incurred in base company currency.
    pub total_base_claimed: Decimal,
    /// Total advance deducted in base company currency.
    pub total_advance_deducted: Decimal,
    /// Net payable amount to employee in base currency (if positive), or refundable to company (if negative).
    pub net_payable_to_employee: Decimal,
}

/// Engine managing employee multi-currency advances and expense claims.
pub struct ExpenseClaimEngine;

impl ExpenseClaimEngine {
    /// Reconciles expense claim details against open employee advances in base currency.
    pub fn settle_claim(
        expenses: &[ExpenseClaimDetail],
        open_advances: &mut [EmployeeAdvance],
    ) -> SettledExpenseClaim {
        let total_base_claimed: Decimal = expenses.iter().map(|e| e.base_claim_amount).sum();
        let mut remaining_to_offset = total_base_claimed;
        let mut total_advance_deducted = Decimal::ZERO;

        for adv in open_advances.iter_mut() {
            if remaining_to_offset <= Decimal::ZERO {
                break;
            }

            if adv.remaining_base_amount <= Decimal::ZERO {
                continue;
            }

            if adv.remaining_base_amount <= remaining_to_offset {
                total_advance_deducted += adv.remaining_base_amount;
                remaining_to_offset -= adv.remaining_base_amount;
                adv.remaining_base_amount = Decimal::ZERO;
            } else {
                adv.remaining_base_amount -= remaining_to_offset;
                total_advance_deducted += remaining_to_offset;
                remaining_to_offset = Decimal::ZERO;
            }
        }

        let net_payable_to_employee = total_base_claimed - total_advance_deducted;

        SettledExpenseClaim {
            total_base_claimed,
            total_advance_deducted,
            net_payable_to_employee,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_multi_currency_advance_claim_offset() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();

        // Advance given: 1,000 EUR @ 1.10 = $1,100 USD base
        let mut advances = vec![EmployeeAdvance {
            name: "ADV-001".into(),
            employee_id: "EMP-001".into(),
            posting_date: date,
            currency: "EUR".into(),
            advance_amount: dec!(1000.0),
            exchange_rate: dec!(1.10),
            base_advance_amount: dec!(1100.0),
            remaining_base_amount: dec!(1100.0),
        }];

        // Expense incurred: 1,500 USD
        let expenses = vec![ExpenseClaimDetail {
            expense_type: "Hotel & Airfare".into(),
            expense_date: date,
            currency: "USD".into(),
            claim_amount: dec!(1500.0),
            exchange_rate: dec!(1.0),
            base_claim_amount: dec!(1500.0),
        }];

        let settlement = ExpenseClaimEngine::settle_claim(&expenses, &mut advances);

        assert_eq!(settlement.total_base_claimed, dec!(1500.0));
        assert_eq!(settlement.total_advance_deducted, dec!(1100.0));
        assert_eq!(settlement.net_payable_to_employee, dec!(400.0)); // Company pays remaining $400
        assert_eq!(advances[0].remaining_base_amount, dec!(0.0)); // Advance fully utilized
    }
}
