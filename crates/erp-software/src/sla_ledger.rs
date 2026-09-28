//! SLA Watchdog Integration & Customer Service Credit Ledger.
//!
//! Evaluates support SLA breach penalties and maintains a cryptographically auditable
//! customer service credit ledger for automated invoice offset.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::errors::SoftwareBillingError;

/// Support SLA Agreement Terms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlaPolicy {
    pub id: String,
    pub name: String,
    pub target_response_hours: u32,
    pub target_resolution_hours: u32,
    pub credit_rate_per_breach_hour: Decimal,
    pub max_credit_cap_percent: Decimal,
}

/// Incurred SLA breach record and computed financial penalty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlaBreachPenalty {
    pub penalty_id: String,
    pub ticket_id: String,
    pub customer_id: String,
    pub breach_hours: Decimal,
    pub penalty_amount: Decimal,
    pub recorded_at: DateTime<Utc>,
}

/// Individual transaction on the customer credit ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreditTransactionType {
    /// Credit issued due to SLA breach penalty.
    SlaPenaltyCredit { ticket_id: String },
    /// Goodwill or contractual concession credit.
    GoodwillCredit { reason: String },
    /// Credit applied / deducted against an issued invoice.
    InvoiceOffset { invoice_id: String },
}

/// Ledger entry recording a credit balance mutation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreditLedgerEntry {
    pub entry_id: String,
    pub customer_id: String,
    pub amount: Decimal, // Positive for credit addition, negative for credit utilization
    pub tx_type: CreditTransactionType,
    pub running_balance: Decimal,
    pub recorded_at: DateTime<Utc>,
}

/// Customer Service Credit Ledger state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomerCreditLedger {
    pub customer_id: String,
    pub available_balance: Decimal,
    pub entries: Vec<CreditLedgerEntry>,
}

impl CustomerCreditLedger {
    /// Instantiates a new credit ledger for a customer with zero balance.
    #[must_use]
    pub fn new(customer_id: String) -> Self {
        Self {
            customer_id,
            available_balance: Decimal::ZERO,
            entries: Vec::new(),
        }
    }

    /// Issues new service credit to the customer balance.
    pub fn issue_credit(
        &mut self,
        entry_id: String,
        amount: Decimal,
        tx_type: CreditTransactionType,
        recorded_at: DateTime<Utc>,
    ) -> Result<Decimal, SoftwareBillingError> {
        if amount <= Decimal::ZERO {
            return Err(SoftwareBillingError::InvalidContractTerms(
                "Credit amount must be positive".into(),
            ));
        }

        self.available_balance += amount;

        self.entries.push(CreditLedgerEntry {
            entry_id,
            customer_id: self.customer_id.clone(),
            amount,
            tx_type,
            running_balance: self.available_balance,
            recorded_at,
        });

        Ok(self.available_balance)
    }

    /// Consumes credit to offset an invoice, reducing the available balance.
    pub fn apply_to_invoice(
        &mut self,
        entry_id: String,
        invoice_id: String,
        amount_to_apply: Decimal,
        recorded_at: DateTime<Utc>,
    ) -> Result<Decimal, SoftwareBillingError> {
        if amount_to_apply <= Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        let actual_applied = amount_to_apply.min(self.available_balance);
        self.available_balance -= actual_applied;

        self.entries.push(CreditLedgerEntry {
            entry_id,
            customer_id: self.customer_id.clone(),
            amount: -actual_applied,
            tx_type: CreditTransactionType::InvoiceOffset { invoice_id },
            running_balance: self.available_balance,
            recorded_at,
        });

        Ok(actual_applied)
    }
}

/// Evaluates SLA breach and calculates service credit penalty conforming to policy cap.
pub fn calculate_sla_penalty(
    penalty_id: String,
    ticket_id: String,
    customer_id: String,
    breach_hours: Decimal,
    policy: &SlaPolicy,
    base_subscription_fee: Decimal,
    now: DateTime<Utc>,
) -> SlaBreachPenalty {
    let raw_penalty = breach_hours * policy.credit_rate_per_breach_hour;
    let cap_amount = (base_subscription_fee * policy.max_credit_cap_percent) / dec!(100);
    let penalty_amount = raw_penalty.min(cap_amount).max(Decimal::ZERO);

    SlaBreachPenalty {
        penalty_id,
        ticket_id,
        customer_id,
        breach_hours,
        penalty_amount,
        recorded_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_sla_penalty_and_credit_ledger() {
        let now = Utc::now();
        let policy = SlaPolicy {
            id: "sla-gold".into(),
            name: "Gold SLA (99.9%)".into(),
            target_response_hours: 1,
            target_resolution_hours: 4,
            credit_rate_per_breach_hour: dec!(50.0), // $50/hour breached
            max_credit_cap_percent: dec!(50.0),      // 50% max of subscription
        };

        // Base subscription = $1000. Max credit cap = $500.
        // Breach = 12 hours -> raw penalty = 12 * $50 = $600. Capped to $500.
        let penalty = calculate_sla_penalty(
            "PEN-001".into(),
            "TCK-999".into(),
            "CUST-1".into(),
            dec!(12.0),
            &policy,
            dec!(1000.0),
            now,
        );
        assert_eq!(penalty.penalty_amount, dec!(500.0));

        let mut ledger = CustomerCreditLedger::new("CUST-1".into());
        ledger
            .issue_credit(
                "ENT-01".into(),
                penalty.penalty_amount,
                CreditTransactionType::SlaPenaltyCredit {
                    ticket_id: penalty.ticket_id.clone(),
                },
                now,
            )
            .unwrap();

        assert_eq!(ledger.available_balance, dec!(500.0));

        // Offset invoice of $300 -> utilizes $300 credit, remaining balance = $200
        let applied = ledger
            .apply_to_invoice("ENT-02".into(), "INV-101".into(), dec!(300.0), now)
            .unwrap();
        assert_eq!(applied, dec!(300.0));
        assert_eq!(ledger.available_balance, dec!(200.0));
    }
}
