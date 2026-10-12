//! Pluggable Payment Gateway Processor & Webhook Settlement Engine (`erp-accounting::payment_gateway`).
//!
//! Provides enterprise e-commerce and POS payment processing with automated
//! double-entry ledger reconciliation upon settlement.

use crate::ledger::{JournalEntry, JournalEntryLine};
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Payment processing status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Authorized,
    Captured,
    Failed,
    Refunded,
}

/// Supported payment methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard,
    BankTransfer,
    DigitalWallet,
    CashOnDelivery,
}

/// Strongly typed payment errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PaymentError {
    #[error("Charge amount must be positive, got {0}")]
    InvalidAmount(Decimal),
    #[error("Payment intent {0} already settled or in terminal state {1:?}")]
    TerminalState(CompactString, PaymentStatus),
    #[error("Gateway authorization rejected: {0}")]
    AuthorizationFailed(CompactString),
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch {
        expected: CompactString,
        actual: CompactString,
    },
}

/// Active payment intent transaction record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentIntent {
    pub intent_id: CompactString,
    pub reference_doctype: CompactString,
    pub reference_docname: CompactString,
    pub customer_id: CompactString,
    pub currency: CompactString,
    pub amount: Decimal,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub gateway_reference: Option<CompactString>,
    pub created_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
}

/// Pluggable payment processor interface.
pub trait PaymentGatewayProcessor: Send + Sync {
    /// Initiates or authorizes a payment intent.
    fn authorize(
        &self,
        intent: &mut PaymentIntent,
        token_or_auth: &str,
    ) -> Result<CompactString, PaymentError>;

    /// Captures previously authorized funds and returns settled gateway ID.
    fn capture(
        &self,
        intent: &mut PaymentIntent,
    ) -> Result<CompactString, PaymentError>;

    /// Refunds a settled transaction.
    fn refund(
        &self,
        intent: &mut PaymentIntent,
        refund_amount: Decimal,
    ) -> Result<CompactString, PaymentError>;
}

/// Enterprise settlement reconciler creating balanced GL entries.
pub struct PaymentReconciliationEngine;

impl PaymentReconciliationEngine {
    /// Synthesizes double-entry General Ledger reconciliation upon payment capture:
    /// - Debit: Bank / Payment Clearing Account
    /// - Credit: Accounts Receivable (Customer Debtor)
    pub fn create_settlement_journal_entry(
        intent: &PaymentIntent,
        bank_account: &str,
        ar_account: &str,
        posting_date: chrono::NaiveDate,
    ) -> Result<JournalEntry, PaymentError> {
        if intent.status != PaymentStatus::Captured {
            return Err(PaymentError::TerminalState(
                intent.intent_id.clone(),
                intent.status,
            ));
        }

        let lines = vec![
            JournalEntryLine {
                account: bank_account.to_string(),
                debit: intent.amount,
                credit: Decimal::ZERO,
                debit_in_account_currency: intent.amount,
                credit_in_account_currency: Decimal::ZERO,
                exchange_rate: Decimal::ONE,
                party_type: None,
                party: None,
            },
            JournalEntryLine {
                account: ar_account.to_string(),
                debit: Decimal::ZERO,
                credit: intent.amount,
                debit_in_account_currency: Decimal::ZERO,
                credit_in_account_currency: intent.amount,
                exchange_rate: Decimal::ONE,
                party_type: Some("Customer".to_string()),
                party: Some(intent.customer_id.to_string()),
            },
        ];

        Ok(JournalEntry {
            posting_date,
            company: "Default Company".to_string(),
            lines,
            remarks: format!(
                "Auto-reconciled payment settlement for {} {}",
                intent.reference_doctype, intent.reference_docname
            ),
        })
    }
}

/// Default in-memory / simulated payment gateway for testing and webhook integration.
#[derive(Debug, Default)]
pub struct MockPaymentGateway {
    pub should_fail: bool,
}

impl MockPaymentGateway {
    #[must_use]
    pub fn new() -> Self {
        Self { should_fail: false }
    }
}

impl PaymentGatewayProcessor for MockPaymentGateway {
    fn authorize(
        &self,
        intent: &mut PaymentIntent,
        token: &str,
    ) -> Result<CompactString, PaymentError> {
        if self.should_fail || token == "tok_fail" {
            intent.status = PaymentStatus::Failed;
            return Err(PaymentError::AuthorizationFailed("Card declined or invalid token".into()));
        }
        if intent.amount <= Decimal::ZERO {
            return Err(PaymentError::InvalidAmount(intent.amount));
        }

        let auth_id: CompactString = format!("AUTH_MOCK_{}", intent.intent_id).into();
        intent.status = PaymentStatus::Authorized;
        intent.gateway_reference = Some(auth_id.clone());
        Ok(auth_id)
    }

    fn capture(&self, intent: &mut PaymentIntent) -> Result<CompactString, PaymentError> {
        if intent.status != PaymentStatus::Authorized {
            return Err(PaymentError::TerminalState(intent.intent_id.clone(), intent.status));
        }

        let capture_id: CompactString = format!("CAP_MOCK_{}", intent.intent_id).into();
        intent.status = PaymentStatus::Captured;
        intent.gateway_reference = Some(capture_id.clone());
        intent.settled_at = Some(Utc::now());
        Ok(capture_id)
    }

    fn refund(
        &self,
        intent: &mut PaymentIntent,
        refund_amount: Decimal,
    ) -> Result<CompactString, PaymentError> {
        if intent.status != PaymentStatus::Captured {
            return Err(PaymentError::TerminalState(intent.intent_id.clone(), intent.status));
        }
        if refund_amount <= Decimal::ZERO || refund_amount > intent.amount {
            return Err(PaymentError::InvalidAmount(refund_amount));
        }

        let ref_id: CompactString = format!("REF_MOCK_{}", intent.intent_id).into();
        intent.status = PaymentStatus::Refunded;
        intent.gateway_reference = Some(ref_id.clone());
        Ok(ref_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_intent_authorization_capture_and_reconciliation() {
        let mut intent = PaymentIntent {
            intent_id: "PI-2026-9001".into(),
            reference_doctype: "Sales Invoice".into(),
            reference_docname: "ACC-SINV-2026-00042".into(),
            customer_id: "CUST-007".into(),
            currency: "USD".into(),
            amount: dec!(56939.50),
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Pending,
            gateway_reference: None,
            created_at: Utc::now(),
            settled_at: None,
        };

        let gateway = MockPaymentGateway::new();
        let auth_res = gateway.authorize(&mut intent, "tok_valid_visa");
        assert!(auth_res.is_ok());
        assert_eq!(intent.status, PaymentStatus::Authorized);

        let cap_res = gateway.capture(&mut intent);
        assert!(cap_res.is_ok());
        assert_eq!(intent.status, PaymentStatus::Captured);

        // Verify GL Reconciliation Entry
        let jv = PaymentReconciliationEngine::create_settlement_journal_entry(
            &intent,
            "1110 - Operating Bank Account",
            "1310 - Accounts Receivable",
            NaiveDate::from_ymd_opt(2026, 10, 12).unwrap(),
        )
        .expect("Reconciliation Journal Entry");

        assert_eq!(jv.lines.len(), 2);
        assert_eq!(jv.lines[0].debit, dec!(56939.50));
        assert_eq!(jv.lines[0].credit, dec!(0.00));
        assert_eq!(jv.lines[1].debit, dec!(0.00));
        assert_eq!(jv.lines[1].credit, dec!(56939.50));
    }
}
